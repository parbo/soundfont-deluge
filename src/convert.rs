use crate::deluge;
use crate::soundfont::{Generator, LoopMode, SoundFont, Unit};
use log::{info, warn};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

macro_rules! get_zone_generator {
    ($zone:expr, $pattern:pat) => ({
        let mut ret = None;
	for g in $zone {
            if let $pattern = g {
                ret = Some(*g);
		break;
            }
        }
        ret
    } as Option<Generator>)
}

pub fn soundfont_to_deluge(
    sf: &SoundFont,
    sample_folder: &Path,
    ix: usize,
    prefix: &str,
) -> deluge::Sound {
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
    for bag_ix in bag_start..bag_end {
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
    let mut attack_vol = vec![];
    let mut decay_vol = vec![];
    let mut sustain_vol = vec![];
    let mut release_vol = vec![];
    loop {
        let mut osc = (LoopMode::NoLoop, vec![]);
        // Find next adjacent
        loop {
            let mut found = false;
            for (zone_ix, zone) in zones.iter().enumerate() {
                if taken.contains(&zone_ix) {
                    continue;
                }
                if let Some(g) = get_zone_generator!(zone, Generator::AttackVolEnv(_)) {
                    if let Some(Unit::Seconds(s)) = g.value() {
                        attack_vol.push(s);
                    }
                }
                if let Some(g) = get_zone_generator!(zone, Generator::DecayVolEnv(_)) {
                    if let Some(Unit::Seconds(s)) = g.value() {
                        decay_vol.push(s);
                    }
                }
                if let Some(g) = get_zone_generator!(zone, Generator::SustainVolEnv(_)) {
                    if let Some(Unit::Level(lvl)) = g.value() {
                        sustain_vol.push(lvl);
                    }
                }
                if let Some(g) = get_zone_generator!(zone, Generator::ReleaseVolEnv(_)) {
                    if let Some(Unit::Seconds(s)) = g.value() {
                        release_vol.push(s);
                    }
                }
                if let Some(Generator::SampleModes(loop_mode)) = get_zone_generator!(zone, Generator::SampleModes(_)) {
                    osc.0 = loop_mode;
                }
                if let Some(Generator::KeyRange(low, high)) = get_zone_generator!(zone, Generator::KeyRange(_, _)) {
                    let mut sample_name = None;
                    if let Some(Generator::SampleID(sample_id)) = get_zone_generator!(zone, Generator::SampleID(_)) {
                        let sample = &sf.samples[sample_id as usize];
                        sample_name = Some(sample.name.clone());
                    }
                    // TODO: check fine tune too
                    let mut root_note = None;
                    if let Some(Generator::OverridingRootKey(root)) =
                        get_zone_generator!(zone, Generator::OverridingRootKey(_))
                    {
                        root_note = Some(root);
                    }
                    if osc.1.is_empty() {
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
        if osc.1.is_empty() {
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
        ix += 1;
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
                get_zone_generator!(&zones[*o], Generator::OverridingRootKey(_))
            {
                if single_sample {
                    osc_builder.transpose(Some((60 - root).into()));
                } else {
                    sample_range_builder.transpose(Some((60 - root).into()));
                }
            }
            if let Some(Generator::FineTune(cents)) = get_zone_generator!(&zones[*o], Generator::FineTune(_)) {
                if single_sample {
                    osc_builder.cents(Some(cents.into()));
                } else {
                    sample_range_builder.cents(Some(cents.into()));
                }
            }
            if let Some(Generator::SampleID(sample_id)) = get_zone_generator!(&zones[*o], Generator::SampleID(_)) {
                let sample = &sf.samples[sample_id as usize];
                let name = format!("{} - {}.wav", sample_id, SoundFont::safe_name(&sample.name));
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
    println!(
        "attack: {} s",
        attack_vol.iter().sum::<f32>() / attack_vol.len() as f32
    );
    println!(
        "decay: {} s",
        decay_vol.iter().sum::<f32>() / decay_vol.len() as f32
    );
    println!(
        "sustain: {} dB",
        sustain_vol.iter().sum::<f32>() / sustain_vol.len() as f32
    );
    println!(
        "release: {} s",
        release_vol.iter().sum::<f32>() / release_vol.len() as f32
    );
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
    let mut preset_name = prefix.to_owned();
    preset_name.push_str(&preset.name);
    sound_builder.name(preset.name.to_owned());
    sound_builder.build().unwrap()
}

pub fn save_deluge_as_xml(sound: &deluge::Sound, folder: &Path) {
    let xml = sound.to_xml();
    fs::create_dir_all(folder).unwrap();
    let file_name = SoundFont::safe_name(&sound.name) + ".xml";
    fs::write(folder.join(Path::new(&file_name)), xml).unwrap();
}

pub fn save_as_xml(sf: &SoundFont, folder: &Path, sample_folder: &Path, ix: usize, prefix: &str) {
    info!("Writing xml to {} for {}", folder.display(), ix);
    let sound = soundfont_to_deluge(sf, sample_folder, ix, prefix);
    save_deluge_as_xml(&sound, folder);
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
    for bag_ix in bag_start..bag_end {
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
