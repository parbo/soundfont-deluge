use binread::*;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::fs;
use std::io::Cursor;
use std::path::Path;

fn make_string(s: &[u8; 20]) -> String {
    let first_null = s.iter().position(|&x| x == 0).unwrap_or(20);
    std::str::from_utf8(&s[0..first_null])
        .unwrap_or("<invalid>")
        .trim()
        .to_string()
}

const RIFF: [u8; 4] = [b'R', b'I', b'F', b'F'];
const PRG: [u8; 4] = [b'p', b'r', b'g', b' '];
const OUT: [u8; 4] = [b'o', b'u', b't', b' '];
const TUNE: [u8; 4] = [b't', b'u', b'n', b'e'];
const LFO: [u8; 4] = [b'l', b'f', b'o', b' '];
const MODS: [u8; 4] = [b'm', b'o', b'd', b's'];
const KGRP: [u8; 4] = [b'k', b'g', b'r', b'p'];
const KLOC: [u8; 4] = [b'k', b'l', b'o', b'c'];
const ENV: [u8; 4] = [b'e', b'n', b'v', b' '];
const FILT: [u8; 4] = [b'f', b'i', b'l', b't'];
const ZONE: [u8; 4] = [b'z', b'o', b'n', b'e'];

#[derive(BinRead, Debug)]
pub struct Zone {
    #[br(pad_before = 1)]
    num_chars: u8,
    sample_name: [u8;20],
    #[br(pad_before = 12)]
    low_velocity: u8,
    high_velocity: u8,
    fine_tune: i8,
    semitone_tune: i8,
    filter: i8,
    pan_balance: i8,
    playback: u8, // TODO: make enum
    output: u8, // TODO: make enum
    zone_level: i8,
    keyboard_track: u8, // TODO: make enum
    velocity: i16,
}

#[derive(BinRead, Debug)]
pub struct Location {
    #[br(pad_before = 4)]
    low_note: u8,
    high_note: u8,
    semitone_tune: i8,
    fine_tune: i8,
    override_fx: u8,  // TODO: make enum
    fx_send_level: u8,
    pitch_mod_1: i8,
    pitch_mod_2: i8,
    amp_mod: i8,
    zone_xfade: u8, // TODO: make enum
    #[br(pad_after = 1)]
    mute_group: u8,
}

#[derive(BinRead, Debug)]
pub struct Envelope {
    #[br(pad_before = 1)]
    rate_1: u8,
    rate_2: u8,
    rate_3: u8,
    rate_4: u8,
    level_1: u8,
    level_2: u8,
    level_3: u8,
    level_4: u8,
    depth: i8,
    velocity_rate_1: i8,
    #[br(pad_before = 1)]
    keyscale: i8,
    #[br(pad_before = 1)]
    velocity_rate_4: i8,
    off_velocity_rate_4: i8,
    #[br(pad_after = 1)]
    velocity_out_level: i8,
}

impl Envelope {
    fn attack(&self) -> u8 {
	self.rate_1
    }
    fn decay(&self) -> u8 {
	self.rate_3
    }
    fn release(&self) -> u8 {
	self.rate_4
    }
    fn sustain(&self) -> u8 {
	self.level_3
    }
}

#[derive(BinRead, Debug)]
pub struct Filter {
    #[br(pad_before = 1)]
    filter_mode: u8, // TODO: make enum
    cutoff_freq: u8,
    resonance: u8,
    keyboard_track: i8,
    mod_input_1: u8,
    mod_input_2: u8,
    mod_input_3: u8,
    #[br(pad_after = 1)]
    headroom: u8, // TODO: make enum
}

#[derive(BinRead, Debug)]
pub struct Mods {
    #[br(pad_before = 5)]
    amp_mod_1_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    amp_mod_2_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    pan_mod_1_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    pan_mod_2_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    pan_mod_3_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_1_rate_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_1_delay_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_1_depth_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_2_rate_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_2_delay_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    lfo_2_depth_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    pitch_mod_1_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    pitch_mod_2_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    amp_mod_src: u8, // TODO: make enum
    #[br(pad_before = 1)]
    filter_mod_input_1: u8, // TODO: make enum
    #[br(pad_before = 1)]
    filter_mod_input_2: u8, // TODO: make enum
    #[br(pad_before = 1)]
    filter_mod_input_3: u8, // TODO: make enum
}

pub struct KeyGroup {
    location: Location,
    amp_env: Envelope,
    filter_env: Envelope,
    aux_env: Envelope,
    zones: [Zone;4],
}

pub struct AkaiProgram {
    mods: Mods,
    key_groups: Vec<KeyGroup>,
}

impl AkaiProgram {
    pub fn parse_akai_program(file: &mut fs::File) -> AkaiProgram {
	let chunk = riff::Chunk::read(file, 0).unwrap();
        let mut todo = VecDeque::new();
        todo.push_back((chunk, 1));
	let mut zones = vec![];
	let mut mods = None;
        loop {
            match todo.pop_back() {
                Some((c, indent)) => {
                    debug!(
                        "{chr:>indent$}Child: id: {}, len: {}",
                        c.id(),
                        c.len(),
                        indent = 2 * indent,
                        chr = ' '
                    );
                    match c.id().value {
                        RIFF => {
                            for child in c.iter(file) {
                                todo.push_back((child, indent + 1));
                            }
                        }
			KGRP => {
                            for child in c.iter_no_type(file) {
                                todo.push_back((child, indent + 1));
                            }
                        }
                        ZONE => {
                            let data = c.read_contents(file).unwrap();
                            let mut reader = Cursor::new(data);
                            if let Ok(zone) = reader.read_be::<Zone>() {
                                debug!(
                                    "{chr:>indent$}Zone: {}",
                                    make_string(&zone.sample_name),
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                                zones.push(zone);
                            }
                        }
                        MODS => {
                            let data = c.read_contents(file).unwrap();
                            let mut reader = Cursor::new(data);
                            if let Ok(m) = reader.read_be::<Mods>() {
                                debug!(
                                    "{chr:>indent$}Mods: {:?}",
                                    m,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
				assert!(mods.is_none());
				mods = Some(m);
                            }
                        }
                        FILT => {
                            let data = c.read_contents(file).unwrap();
                            let mut reader = Cursor::new(data);
                            if let Ok(filter) = reader.read_be::<Filter>() {
                                debug!(
                                    "{chr:>indent$}Filter: {:?}",
                                    filter,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                        }
                        ENV => {
                            let data = c.read_contents(file).unwrap();
                            let mut reader = Cursor::new(data);
                            if let Ok(env) = reader.read_be::<Envelope>() {
                                debug!(
                                    "{chr:>indent$}Envelope: {:?}",
                                    env,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                        }
                        _ => {
                        }
                    }
                }
                None => break,
            }
        }

        AkaiProgram {
	    mods: mods.unwrap(),
	    key_groups: vec![],
        }
    }

    pub fn dump(&self) {
    }

    pub fn safe_name(s: &str) -> String {
        s.chars()
            .map(|x| match x {
                '/' => '_',  // filesystem
                '\\' => '_', // filesystem
                '?' => '_',  // filesystem
                '*' => '_',  // filesystem
                '\'' => '_', // xml
                '"' => '_',  // xml
                '<' => '_',  // xml
                '>' => '_',  // xml
                '&' => '_',  // xml
                _ => x,
            })
            .collect()
    }
}
