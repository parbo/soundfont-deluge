extern crate yaserde;
#[macro_use]
extern crate yaserde_derive;

mod deluge;
mod soundfont;

use clap::{App, Arg};
use log::{info, warn};
use soundfont::{Generator, LoopMode, SoundFont};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn save_as_xml(sf: &SoundFont, folder: &Path, sample_folder: &Path, ix: usize, prefix: &str) {
    info!("Writing xml to {} for {}", folder.display(), ix);
    let is_last = ix == sf.presets.len() - 1;
    let preset = &sf.presets[ix];
    info!("Preset: {}", preset.name);
    let bag_start = preset.bag_index as usize;
    let bag_end = if is_last {
        sf.pbags.len()
    } else {
        let next_preset = &sf.presets[ix + 1];
        next_preset.bag_index as usize
    };
    let mut zones = vec![];
    let mut zone = 0;
    for bag_ix in bag_start..bag_end {
        zone = zone + 1;
        let is_last = ix == sf.pbags.len() - 1;
        let bag = &sf.pbags[bag_ix];
        let gen_start = bag.gen_ndx as usize;
        let gen_end = if is_last {
            sf.pgens.len()
        } else {
            let next_bag = &sf.pbags[bag_ix + 1];
            next_bag.gen_ndx as usize
        };
        for gen_ix in gen_start..gen_end {
            let gen = &sf.pgens[gen_ix];
            if let Generator::Instrument(index) = gen {
                let mut gens = get_instrument_zones(sf, *index as usize);
                zones.append(&mut gens);
            }
        }
    }
    // Map zones to oscs
    let mut oscs = vec![];
    let mut taken = HashSet::new();
    loop {
        let mut osc = (LoopMode::NoLoop, vec![]);
        // Find next adjacent
        loop {
            let mut found = false;
            for zone_ix in 0..zones.len() {
                if taken.contains(&zone_ix) {
                    continue;
                }
                let zone = &zones[zone_ix];
                if let Some(Generator::SampleModes(loop_mode)) = get_zone_sample_mode(zone) {
                    osc.0 = loop_mode;
                }
                if let Some(Generator::KeyRange(low, high)) = get_zone_key_range(zone) {
                    let mut sample_name = None;
                    if let Some(Generator::SampleID(sample_id)) = get_zone_sample(zone) {
                        let sample = &sf.samples[sample_id as usize];
                        sample_name = Some(sample.name.clone());
                    }
                    // TODO: check fine tune too
                    let mut root_note = None;
                    if let Some(Generator::OverridingRootKey(root)) =
                        get_zone_overriding_root_key(zone)
                    {
                        root_note = Some(root);
                    }
                    if osc.1.len() == 0 {
                        osc.1.push((zone_ix, low, high, sample_name, root_note));
                        taken.insert(zone_ix);
                        found = true;
                    } else {
                        let (_prev_zone, _prev_low, prev_high, prev_sample_name, prev_root_note) =
                            osc.1.last_mut().unwrap();
                        if *prev_high + 1 == low {
                            taken.insert(zone_ix);
                            found = true;
                            if sample_name != *prev_sample_name || root_note != *prev_root_note {
                                // Add the new sample
                                osc.1.push((zone_ix, low, high, sample_name, root_note));
                            } else {
                                // Just extend previous range. In soundfonts, each range can have different params, but in deluge they can't.
                                *prev_high = high;
                            }
                        }
                    }
                }
            }
            if !found {
                break;
            }
        }
        if osc.1.len() == 0 {
            break;
        }
        info!("osc: {:?}", osc);
        oscs.push(osc);
    }

    // Write out the first two
    let mut sound_builder = deluge::SoundBuilder::default();
    sound_builder.firmware_version(Some("3.1.3".to_string()));
    sound_builder.earliest_compatible_firmware(Some("3.1.0-beta".to_string()));
    let mut default_params_builder = deluge::DefaultParamsBuilder::default();
    let mut ix = 0;
    let num = oscs.len();
    if num > 2 {
        warn!(
            "{} has more osc than the deluge has, need to select",
            preset.name
        );
    }
    for (loop_mode, osc) in &oscs[0..std::cmp::min(num, 2)] {
        ix = ix + 1;
        let mut osc_builder = deluge::OscBuilder::default();
        osc_builder
            .osc_type(deluge::OscType::Sample)
            .transpose(None)
            .cents(None)
            .retrig_phase(None)
            .reversed(Some(0))
            .time_stretch_enable(Some(0))
            .time_stretch_amount(Some(0))
            .loop_mode(Some(0)); // Always use loop mode 0 (Cut)
        let single_sample = osc.len() == 1;
        let mut sample_ranges = vec![];
        for (ix, (o, _low, high, _sample_name, _root)) in osc.iter().enumerate() {
            let mut sample_range_builder = deluge::SampleRangeBuilder::default();
            // The last sample must _not_ have range_top_note!
            if ix != osc.len() - 1 {
                sample_range_builder.range_top_note(Some(*high as i32));
            }
            if let Some(Generator::OverridingRootKey(root)) =
                get_zone_overriding_root_key(&zones[*o])
            {
                if single_sample {
                    osc_builder.transpose(Some((60 - root).into()));
                } else {
                    sample_range_builder.transpose(Some((60 - root).into()));
                }
            }
            if let Some(Generator::FineTune(cents)) = get_zone_fine_tune(&zones[*o]) {
                if single_sample {
                    osc_builder.cents(Some(cents.into()));
                } else {
                    sample_range_builder.cents(Some(cents.into()));
                }
            }
            if let Some(Generator::SampleID(sample_id)) = get_zone_sample(&zones[*o]) {
                let sample = &sf.samples[sample_id as usize];
                let name = SoundFont::safe_name(&sample.name) + ".wav";
                let file_path: Vec<String> = sample_folder
                    .join(name)
                    .components()
                    .map(|x| x.as_os_str().to_str().unwrap().into())
                    .collect();
                if single_sample {
                    osc_builder.file_name(Some(file_path.join("/")));
                } else {
                    sample_range_builder.file_name(Some(file_path.join("/")));
                }
                // TODO: take generator sample offsets into account
                let mut zone_builder = deluge::ZoneBuilder::default();
                zone_builder.end_sample_pos(sample.end - sample.start);
                if *loop_mode != LoopMode::NoLoop {
                    zone_builder.start_loop_pos(Some(sample.start_loop - sample.start));
                    zone_builder.end_loop_pos(Some(sample.end_loop - sample.start));
                }
                if single_sample {
                    osc_builder.zone(Some(zone_builder.build().unwrap()));
                } else {
                    sample_range_builder.zone(zone_builder.build().unwrap());
                }
            }
            if !single_sample {
                let sample_range = sample_range_builder.build().unwrap();
                sample_ranges.push(sample_range);
            }
        }
        if !single_sample {
            osc_builder.sample_ranges(Some(
                deluge::SampleRangesBuilder::default()
                    .sample_range(sample_ranges)
                    .build()
                    .unwrap(),
            ));
        }
        let osc = osc_builder.build().unwrap();
        if ix == 1 {
            sound_builder.osc1(osc);
            default_params_builder.osc1_volume(deluge::Value(0x7FFFFFFF));
        } else {
            sound_builder.osc2(osc);
            default_params_builder.osc2_volume(deluge::Value(0x7FFFFFFF));
        }
    }
    // Set the amp envelope to have attack 50, decay 25, sustain 50, release 25
    default_params_builder.envelope1(
        deluge::EnvelopeBuilder::default()
            .attack(deluge::Value(0x80000000))
            .decay(deluge::Value(0x00000000))
            .sustain(deluge::Value(0x7FFFFFD2))
            .release(deluge::Value(0x00000000))
            .build()
            .unwrap(),
    );
    sound_builder.default_params(default_params_builder.build().unwrap());
    let sound = sound_builder.build().unwrap();

    let xml = sound.to_xml();
    fs::create_dir_all(folder).unwrap();
    let mut preset_name = prefix.to_owned();
    preset_name.push_str(&preset.name);
    let file_name = SoundFont::safe_name(&preset_name) + ".xml";
    fs::write(folder.join(Path::new(&file_name)), xml).unwrap();
}

fn get_zone_sample(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::SampleID(_) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_sample_mode(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::SampleModes(_) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_key_range(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::KeyRange(_, _) = g {
            return Some(*g);
        }
    }
    None
}

// fn get_zone_vel_range(zone: &[Generator]) -> Option<Generator> {
//     for g in zone {
//         if let Generator::VelRange(_, _) = g {
//             return Some(*g);
//         }
//     }
//     None
// }

// fn get_zone_attack(zone: &[Generator]) -> Option<Generator> {
//     for g in zone {
//         if let Generator::AttackVolEnv(_) = g {
//             return Some(*g);
//         }
//     }
//     None
// }

// fn get_zone_decay(zone: &[Generator]) -> Option<Generator> {
//     for g in zone {
//         if let Generator::DecayVolEnv(_) = g {
//             return Some(*g);
//         }
//     }
//     None
// }

fn get_zone_overriding_root_key(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::OverridingRootKey(_) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_fine_tune(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::FineTune(_) = g {
            return Some(*g);
        }
    }
    None
}

fn get_instrument_zones(sf: &SoundFont, ix: usize) -> Vec<Vec<Generator>> {
    let mut zones = vec![];
    let is_last = ix == sf.instruments.len() - 1;
    let instrument = &sf.instruments[ix];
    let bag_start = instrument.bag_index as usize;
    let bag_end = if is_last {
        sf.ibags.len()
    } else {
        let next_instrument = &sf.instruments[ix + 1];
        next_instrument.bag_index as usize
    };
    let mut zone = 0;
    for bag_ix in bag_start..bag_end {
        zone = zone + 1;
        let is_last = ix == sf.ibags.len() - 1;
        let bag = &sf.ibags[bag_ix];
        let gen_start = bag.gen_ndx as usize;
        let gen_end = if is_last {
            sf.igens.len()
        } else {
            let next_bag = &sf.ibags[bag_ix + 1];
            next_bag.gen_ndx as usize
        };
        let mut zone = vec![];
        for gen_ix in gen_start..gen_end {
            let gen = &sf.igens[gen_ix];
            zone.push(*gen);
        }
        zones.push(zone);
    }
    zones
}

fn main() {
    env_logger::init();

    let matches = App::new("Soundfont => Deluge")
        .version("0.1")
        .author("Pär Bohrarper <par@bohrarper.se>")
        .about("Converts Soundfonts to Deluge xml + sample folders")
        .arg(
            Arg::with_name("INPUT")
                .short("i")
                .long("input")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("SAMPLES")
                .short("a")
                .long("sample-folder")
                .takes_value(true)
                .help("Sets the output folder to save samples to")
                .required(false),
        )
        .arg(
            Arg::with_name("SYNTH")
                .short("y")
                .long("synth-folder")
                .takes_value(true)
                .help("Sets the output folder to save synth xml to")
                .required(false),
        )
        .arg(
            Arg::with_name("PREFIX")
                .short("p")
                .long("synth-prefix")
                .takes_value(true)
                .help("Sets a prefix to prepend to synth xml file names")
                .required(false),
        )
        .arg(
            Arg::with_name("DUMP")
                .help("Dump info")
                .short("d")
                .takes_value(false)
                .required(false),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required.
    let filename = &matches.value_of("INPUT").unwrap();
    let mut file = fs::File::open(Path::new(filename)).unwrap();

    if filename.to_lowercase().ends_with(".xml") {
        let synth = deluge::Sound::from_xml(&mut file);
        if matches.is_present("DUMP") {
            println!("dumping");
            println!("{:?}", synth);
        }
    } else {
        let sf = SoundFont::parse_soundfont(&mut file);
        if matches.is_present("DUMP") {
            println!("dumping");
            sf.dump();
        }
        let sample_folder = matches.value_of("SAMPLES");
        if let Some(folder) = sample_folder {
            sf.save_samples(Path::new(folder)).unwrap();
        }
        if let Some(xml_folder) = matches.value_of("SYNTH") {
            // TODO: save all xmls
            // Note: if the samples aren't saved above we use a dummy folder
            let samples = sample_folder.unwrap_or("SAMPLES");
            let prefix = matches.value_of("PREFIX").unwrap_or("");
            for ix in 0..(sf.presets.len() - 1) {
                save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), ix, &prefix);
            }
            // save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 2, prefix);
            // save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 5, prefix);
            // save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 20, prefix);
            // save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 22, prefix);
        }
    }
}
