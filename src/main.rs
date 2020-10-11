use std::env;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use binread::*;
use std::collections::VecDeque;

fn make_string(s: &[u8;20]) -> String {
    let first_null = s.iter().position(|&x| x == 0).unwrap_or(20);
    std::str::from_utf8(&s[0..first_null]).unwrap_or("<invalid>").trim().to_string()
}

#[derive(BinRead, Debug)]
struct Sample {
    #[br(map = |x: [u8;20]| make_string(&x))]
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
    #[br(map = |x: [u8;20]| make_string(&x))]
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
    #[br(map = |x: [u8;20]| make_string(&x))]
    name : String,
    bag_index: u16,
}

#[derive(BinRead, Debug)]
struct Generator {
    operator: u16,
    amount: u16,
}

// #         Name         Unit         Abs         Zero Min         Min         Useful Max        Max        Useful De-fault Def Value
// 0 startAddrsOffset, +, smpls, 0, 0, None, *, *, 0, None
// 1 endAddrsOffset, +, smpls, 0              *              *              0              None              0              None
// 2 startloopAddrsOffset              +              smpls              0              *              *              *              *              0              None
// 3 endloopAddrsOffset              +              smpls              0              *              *              *              *              0              None
// 4 startAddrsCoarseOffset + 32k smpls 0 0 None * * 0 None
// 5 modLfoToPitch cent fs 0 -12000 -10 oct 12000 10 oct 0 None
// 6 vibLfoToPitch cent fs 0 -12000 -10 oct 12000 10 oct 0 None
// 7 modEnvToPitch cent fs 0 -12000 -10 oct 12000 10 oct 0 None
// 8 initialFilterFc         cent         8.176         Hz 1500 20 Hz 13500 20 kHz 13500 Open
// 9 initialFilterQ       cB       0       0       None       960       96       dB       0       None
// 10 modLfoToFilterFc cent fs 0 -12000 -10 oct 12000 10 oct 0 None
// 11 modEnvToFilterFc cent fs 0 -12000 -10 oct 12000 10 oct 0 None
// 12 endAddrsCoarseOffset              +              32k              smpls              0              *              *              0              None              0              None
// 13 modLfoToVolume cB fs 0 -960 -96 dB 960 96 dB 0 None
// 15 chorusEffectsSend       0.1%       0              0              None              1000              100%              0              None
// 16 reverbEffectsSend       0.1%       0              0              None              1000              100%              0              None
// 17 pan       0.1%       Cntr       -500       Left       +500       Right       0       Center
// 21 delayModLFO timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 msec
// 22 freqModLFO       cent       8.176       Hz -16000 1 mHz 4500 100 Hz 0 8.176 Hz
// 23 delayVibLFO timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 msec
// 24 freqVibLFO       cent       8.176       Hz -16000 1 mHz 4500 100 Hz 0 8.176 Hz
// 25 delayModEnv timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 msec
// 26 attackModEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec
// 27 holdModEnv timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 
// 28 decayModEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec
// 29 sustainModEnv       -0.1%       attk       peak 0              100%              1000              0%              0              attk              pk
// 30 releaseModEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec
// 31 keynumToModEnvHold       tcent/key       0       -1200       -oct/ky       1200       oct/ky       0       None
// 32 keynumToModEnvDecay       tcent/key       0             -1200             -oct/ky             1200             oct/ky             0             None
// 33 delayVolEnv timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 msec
// 34 attackVolEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec
// 35 holdVolEnv timecent 1 sec -12000 1 msec 5000 20 sec -12000 <1 msec
// 36 decayVolEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec 37       sustainVolEnv       cB       attn       attk       peak 0              0              dB              1440              144dB              0              attk              pk
// 38 releaseVolEnv timecent 1 sec -12000 1 msec 8000 100sec -12000 <1 msec
// 39 keynumToVolEnvHold       tcent/key       0       -1200       -oct/ky       1200       oct/ky       0       None
// 40 keynumToVolEnvDecay       tcent/key       0             -1200             -oct/ky             1200             oct/ky             0             None
// 41 instrument
// 43 keyRange @ MIDI ky# key# 0 0 lo key 127 hi key 0-127 full kbd
// 44 velRange @ MIDI vel 0 0 min vel 127         max         vel 0-127      all      vels
// 45 startloopAddrsCoarseOffset              +              smpls              0              *              *              *              *              0              None
// 46 keynum+@       MIDI       ky#       key#       0 0 lo key 127 hi key -1 None
// 47 velocity +@ MIDI vel 0 1 min vel 127         mx         vel         -1         None
// 48 initialAttenuation       cB       0       0              0              dB              1440              144dB              0              None
// 50 endloopAddrsCoarseOffset              +              smpls              0              *              *              *              *              0              None
// 51 coarseTune       semitone       0       -120 -10 oct 120 10 oct 0 None 52       fineTune       cent       0       -99       -99cent 99           99cent           0           None
// 54 sampleModes +@ Bit Flags Flags ** ** ** ** 0 No Loop
// 56 scaleTuning       @       cent/key       0       0       none       1200       oct/ky       100       semi-tone
// 57 exclusiveClass              +@              arbitrary#              0              1              --              127              --              0              None
// 58 overridingRootKey +@ MIDI ky# key# 0 0 lo key 127 hi key -1 None

const RIFF: [u8;4] = [b'R', b'I', b'F', b'F'];
const LIST: [u8;4] = [b'L', b'I', b'S', b'T'];
const INAM: [u8;4] = [b'I', b'N', b'A', b'M'];
const SDTA: [u8;4] = [b's', b'd', b't', b'a'];
const SHDR: [u8;4] = [b's', b'h', b'd', b'r'];
const SMPL: [u8;4] = [b's', b'm', b'p', b'l'];
const PHDR: [u8;4] = [b'p', b'h', b'd', b'r'];
const INST: [u8;4] = [b'i', b'n', b's', b't'];
const IGEN: [u8;4] = [b'i', b'g', b'e', b'n'];

fn parse_soundfont(chunk: riff::Chunk, file: &mut File) {
    let mut todo = VecDeque::new();
    todo.push_back((chunk, 1));
    let mut samples = vec![];
    let mut _sample_data = vec![];
    let mut presets = vec![];
    let mut instruments = vec![];
    let mut generators = vec![];
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
		    _sample_data = c.read_contents(file).unwrap();
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
		IGEN => {
		    let data = c.read_contents(file).unwrap();
		    let mut reader = Cursor::new(data);
		    while let Ok(generator) = reader.read_ne::<Generator>() {
			// println!("{chr:>indent$}Instrument Generator: {}, {}", generator.operator, generator.amount, indent=2 * (indent + 1), chr=' ');
			generators.push(generator);
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
