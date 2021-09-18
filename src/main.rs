mod deluge;
mod soundfont;

use clap::{App, Arg};
use log::info;
use soundfont::{Generator, SoundFont};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn save_as_xml(sf: &SoundFont, folder: &Path, sample_folder: &Path, ix: usize) {
    info!("Writing xml to {} for {}", folder.display(), ix);
    let mut synth_builder = deluge::SynthBuilder::default();
    synth_builder.firmware_version(Some("3.1.3".to_string()));
    synth_builder.earliest_compatible_firmware(Some("3.1.0-beta".to_string()));
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
        let mut osc = vec![];
        // Find next adjacent
        loop {
            let mut found = false;
            for zone_ix in 0..zones.len() {
                if taken.contains(&zone_ix) {
                    continue;
                }
                let zone = &zones[zone_ix];
                if let Some(Generator::KeyRange(low, _high)) = get_zone_key_range(zone) {
                    let mut with_velrange = None;
                    if let Some(Generator::VelRange(low, high)) = get_zone_vel_range(zone) {
                        if low == 0 {
                            with_velrange = Some("low");
                        } else if high == 127 {
                            with_velrange = Some("high");
                        } else {
                            with_velrange = Some("other");
                        }
                    }
                    let mut with_attack = None;
                    if let Some(Generator::AttackVolEnv(vol)) = get_zone_attack(zone) {
                        with_attack = Some(f32::powf(2.0, (vol as f32 - 12000.0) / 1200.0));
                    }
                    let mut with_decay = None;
                    if let Some(Generator::DecayVolEnv(vol)) = get_zone_decay(zone) {
                        with_decay = Some(f32::powf(2.0, (vol as f32 - 12000.0) / 1200.0));
                    }
                    if osc.len() == 0 {
                        osc.push((zone_ix, with_velrange, with_attack, with_decay));
                        taken.insert(zone_ix);
                        found = true;
                    } else {
                        let (prev_zone, _prev_velrange, _prev_attack, _prev_decay) =
                            osc.last().unwrap();
                        if let Some(Generator::KeyRange(_plow, phigh)) =
                            get_zone_key_range(&zones[*prev_zone])
                        {
                            if phigh + 1 == low {
                                osc.push((zone_ix, with_velrange, with_attack, with_decay));
                                taken.insert(zone_ix);
                                found = true;
                            }
                        }
                    }
                }
            }
            if !found {
                break;
            }
        }
        info!("osc: {:?}", osc);
        if osc.len() == 0 {
            break;
        }
        oscs.push(osc);
    }

    // Write out the first two
    let mut sound_builder = deluge::SoundBuilder::default();
    let mut ix = 0;
    let num = oscs.len();
    for osc in &oscs[0..std::cmp::min(num, 2)] {
        ix = ix + 1;
        let mut sample_ranges = vec![];
        for (o, _vel_range, _attack, _decay) in osc {
            let mut sample_range_builder = deluge::SampleRangeBuilder::default();
            if let Some(Generator::KeyRange(_low, high)) = get_zone_key_range(&zones[*o]) {
                sample_range_builder.range_top_note(Some(high.into()));
            }
            if let Some(Generator::OverridingRootKey(root)) =
                get_zone_overriding_root_key(&zones[*o])
            {
                sample_range_builder.transpose(Some((60 - root).into()));
            }
            if let Some(Generator::FineTune(cents)) = get_zone_fine_tune(&zones[*o]) {
                sample_range_builder.cents(Some(cents.into()));
            }
            if let Some(Generator::SampleID(sample_id)) = get_zone_sample(&zones[*o]) {
                let sample = &sf.samples[sample_id as usize];
                let name = SoundFont::safe_name(&sample.name) + ".wav";
                let file_path: Vec<String> = sample_folder
                    .join(name)
                    .components()
                    .map(|x| x.as_os_str().to_str().unwrap().into())
                    .collect();
                sample_range_builder.file_name(file_path.join("/"));
                // TODO: take generator sample offsets into account
                // TODO: use loop points
                sample_range_builder.zone(
                    deluge::ZoneBuilder::default()
                        .end_sample_pos(sample.end - sample.start)
                        .build()
                        .unwrap(),
                );
            }
            let sample_range = sample_range_builder.build().unwrap();
            sample_ranges.push(sample_range);
        }
        let osc = deluge::OscBuilder::default()
            .osc_type(deluge::OscType::Sample)
            .loop_mode(Some(0))
            .reversed(Some(0))
            .time_stretch_enable(Some(0))
            .time_stretch_amount(Some(0))
            .sample_ranges(Some(
                deluge::SampleRangesBuilder::default()
                    .sample_range(sample_ranges)
                    .build()
                    .unwrap(),
            ))
            .build()
            .unwrap();
        if ix == 1 {
            sound_builder.osc1(osc);
        } else {
            sound_builder.osc2(osc);
        }
    }
    synth_builder.sound(sound_builder.build().unwrap());
    let synth = synth_builder.build().unwrap();

    let xml = synth.to_xml();
    fs::create_dir_all(folder).unwrap();
    let file_name = SoundFont::safe_name(&preset.name) + ".xml";
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

fn get_zone_key_range(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::KeyRange(_, _) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_vel_range(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::VelRange(_, _) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_attack(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::AttackVolEnv(_) = g {
            return Some(*g);
        }
    }
    None
}

fn get_zone_decay(zone: &[Generator]) -> Option<Generator> {
    for g in zone {
        if let Generator::DecayVolEnv(_) = g {
            return Some(*g);
        }
    }
    None
}

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
        let synth = deluge::parse_synth(&mut file);
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
            save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 2);
            save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 5);
            save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 20);
            save_as_xml(&sf, Path::new(xml_folder), Path::new(samples), 22);
        }
    }
}
