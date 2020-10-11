use std::env;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use binread::*;
use std::collections::VecDeque;

fn makeString(s: &[u8;20]) -> String {
    let first_null = s.iter().position(|&x| x == 0).unwrap_or(20);
    std::str::from_utf8(&s[0..first_null]).unwrap_or("<invalid>").trim().to_string()
}

#[derive(BinRead, Debug)]
struct Sample {
    #[br(map = |x: [u8;20]| makeString(&x))]
    name : String,
    start: u32,
    end: u32,
    start_loop: u32,
    end_loop: u32,
    sample_rate: u32,
    original_pitch: u8,
    correction: i8,
    sample_link: u16,
    sample_type: u16,
}

#[derive(BinRead, Debug)]
struct Preset {
    #[br(map = |x: [u8;20]| makeString(&x))]
    name : String,
    preset: u16,
    bank: u16,
    bag_index: u16,
    library: u32,
    genre: u32,
    morphology: u32,
}

#[derive(BinRead, Debug)]
struct Instrument {
    #[br(map = |x: [u8;20]| makeString(&x))]
    name : String,
    bag_index: u16,
}

const RIFF: [u8;4] = [b'R', b'I', b'F', b'F'];
const LIST: [u8;4] = [b'L', b'I', b'S', b'T'];
const INAM: [u8;4] = [b'I', b'N', b'A', b'M'];
const SDTA: [u8;4] = [b's', b'd', b't', b'a'];
const SHDR: [u8;4] = [b's', b'h', b'd', b'r'];
const SMPL: [u8;4] = [b's', b'm', b'p', b'l'];
const PHDR: [u8;4] = [b'p', b'h', b'd', b'r'];
const INST: [u8;4] = [b'i', b'n', b's', b't'];

fn parse_soundfont(chunk: riff::Chunk, file: &mut File) {
    let mut todo = VecDeque::new();
    todo.push_back((chunk, 1));
    let mut samples = vec![];
    let mut sample_data = vec![];
    let mut presets = vec![];
    let mut instruments = vec![];
    loop {
	if let Some((c, indent)) = todo.pop_back() {
	    println!("{chr:>indent$}Child: id: {}, len: {}", c.id(), c.len(), indent=2 * indent, chr=' ');
	    match c.id().value {
		RIFF | LIST | SDTA => {
		    for child in c.iter(file) {
			todo.push_back((child, indent + 1));
		    }
		},
		INAM => {
		    let data = c.read_contents(file).unwrap();
		    let name = String::from_utf8(data).unwrap();
		    println!("{chr:>indent$}Name: {}", name, indent=2 * (indent + 1), chr=' ');
		},
		SMPL => {
		    sample_data = c.read_contents(file).unwrap();
		    println!("{chr:>indent$}Samples: {}", c.len() / 2, indent=2 * (indent + 1), chr=' ');
		},
		SHDR => {
		    let data = c.read_contents(file).unwrap();
		    let mut reader = Cursor::new(data);
		    while let Ok(sample) = reader.read_ne::<Sample>() {
			if !sample.name.starts_with("EOS") {
			    println!("{chr:>indent$}Sample: {}", sample.name, indent=2 * (indent + 1), chr=' ');
			    samples.push(sample);
			}
		    }
		},
		PHDR => {
		    let data = c.read_contents(file).unwrap();
		    let mut reader = Cursor::new(data);
		    while let Ok(preset) = reader.read_ne::<Preset>() {
			if !preset.name.starts_with("EOP") {
			    println!("{chr:>indent$}Preset: {}", preset.name, indent=2 * (indent + 1), chr=' ');
			    presets.push(preset);
			}
		    }
		},
		INST => {
		    let data = c.read_contents(file).unwrap();
		    let mut reader = Cursor::new(data);
		    while let Ok(instrument) = reader.read_ne::<Instrument>() {
			if !instrument.name.starts_with("EOI") {
			    println!("{chr:>indent$}Instrument: {}", instrument.name, indent=2 * (indent + 1), chr=' ');
			    instruments.push(instrument);
			}
		    }
		},
		_ => {}
	    }
	} else {
	    break;
	}
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut file = File::open(Path::new(filename)).unwrap();

    let chunk = riff::Chunk::read(&mut file, 0).unwrap();
    parse_soundfont(chunk, &mut file);
}
