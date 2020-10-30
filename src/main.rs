use binread::*;
use log::{debug, info, error};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

fn make_string(s: &[u8; 20]) -> String {
    let first_null = s.iter().position(|&x| x == 0).unwrap_or(20);
    std::str::from_utf8(&s[0..first_null])
        .unwrap_or("<invalid>")
        .trim()
        .to_string()
}

#[derive(BinRead, Debug)]
struct Sample {
    #[br(map = |x: [u8;20]| make_string(&x))]
    name: String,
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
    name: String,
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
    name: String,
    bag_index: u16,
}

#[derive(Debug, PartialEq, Eq)]
enum GeneratorType {
    StartAddrsOffset,
    EndAddrsOffset,
    StartloopAddrsOffset,
    EndloopAddrsOffset,
    StartAddrsCoarseOffset,
    ModLfoToPitch,
    VibLfoToPitch,
    ModEnvToPitch,
    InitialFilterFc,
    InitialFilterQ,
    ModLfoToFilterFc,
    ModEnvToFilterFc,
    EndAddrsCoarseOffset,
    ModLfoToVolume,
    ChorusEffectsSend,
    ReverbEffectsSend,
    Pan,
    DelayModLFO,
    FreqModLFO,
    DelayVibLFO,
    FreqVibLFO,
    DelayModEnv,
    AttackModEnv,
    HoldModEnv,
    DecayModEnv,
    SustainModEnv,
    ReleaseModEnv,
    KeynumToModEnvHold,
    KeynumToModEnvDecay,
    DelayVolEnv,
    SustainVolEnv,
    AttackVolEnv,
    HoldVolEnv,
    DecayVolEnv,
    ReleaseVolEnv,
    KeynumToVolEnvHold,
    KeynumToVolEnvDecay,
    Instrument,
    KeyRange,
    VelRange,
    StartloopAddrsCoarseOffset,
    Keynum,
    Velocity,
    InitialAttenuation,
    EndloopAddrsCoarseOffset,
    CoarseTune,
    FineTune,
    SampleID,
    SampleModes,
    ScaleTuning,
    ExclusiveClass,
    OverridingRootKey,
    EndOper,
    Unused,
}

fn parse_generator(v: u16) -> GeneratorType {
    match v {
        0 => GeneratorType::StartAddrsOffset,
        1 => GeneratorType::EndAddrsOffset,
        2 => GeneratorType::StartloopAddrsOffset,
        3 => GeneratorType::EndloopAddrsOffset,
        4 => GeneratorType::StartAddrsCoarseOffset,
        5 => GeneratorType::ModLfoToPitch,
        6 => GeneratorType::VibLfoToPitch,
        7 => GeneratorType::ModEnvToPitch,
        8 => GeneratorType::InitialFilterFc,
        9 => GeneratorType::InitialFilterQ,
        10 => GeneratorType::ModLfoToFilterFc,
        11 => GeneratorType::ModEnvToFilterFc,
        12 => GeneratorType::EndAddrsCoarseOffset,
        13 => GeneratorType::ModLfoToVolume,
        15 => GeneratorType::ChorusEffectsSend,
        16 => GeneratorType::ReverbEffectsSend,
        17 => GeneratorType::Pan,
        21 => GeneratorType::DelayModLFO,
        22 => GeneratorType::FreqModLFO,
        23 => GeneratorType::DelayVibLFO,
        24 => GeneratorType::FreqVibLFO,
        25 => GeneratorType::DelayModEnv,
        26 => GeneratorType::AttackModEnv,
        27 => GeneratorType::HoldModEnv,
        28 => GeneratorType::DecayModEnv,
        29 => GeneratorType::SustainModEnv,
        30 => GeneratorType::ReleaseModEnv,
        31 => GeneratorType::KeynumToModEnvHold,
        32 => GeneratorType::KeynumToModEnvDecay,
        33 => GeneratorType::DelayVolEnv,
        34 => GeneratorType::AttackVolEnv,
        35 => GeneratorType::HoldVolEnv,
        36 => GeneratorType::DecayVolEnv,
	37 => GeneratorType::SustainVolEnv,
        38 => GeneratorType::ReleaseVolEnv,
        39 => GeneratorType::KeynumToVolEnvHold,
        40 => GeneratorType::KeynumToVolEnvDecay,
        41 => GeneratorType::Instrument,
        43 => GeneratorType::KeyRange,
        44 => GeneratorType::VelRange,
        45 => GeneratorType::StartloopAddrsCoarseOffset,
        46 => GeneratorType::Keynum,
        47 => GeneratorType::Velocity,
        48 => GeneratorType::InitialAttenuation,
        49 => GeneratorType::EndloopAddrsCoarseOffset,
        51 => GeneratorType::CoarseTune,
	52 => GeneratorType::FineTune,
        53 => GeneratorType::SampleID,
        54 => GeneratorType::SampleModes,
        56 => GeneratorType::ScaleTuning,
        57 => GeneratorType::ExclusiveClass,
        58 => GeneratorType::OverridingRootKey,
	60 => GeneratorType::EndOper,
        _x => {
	    error!("Ununsed generator: {}", _x);
	    GeneratorType::Unused
	}
    }
}

#[derive(BinRead, Debug)]
struct Generator {
    #[br(map = |x: u16| parse_generator(x))]
    oper: GeneratorType,
    amount: u16,
}

#[derive(Debug, PartialEq, Eq)]
enum SourceEnumeratorController {
    NoController,
    NoteOnVelocity,
    NoteOnKeyNumber,
    PolyPressure,
    ChannelPressure,
    PitchWheel,
    PitchWheelSensitivity,
    Link,
    MidiCC(u8),
}

#[derive(Debug, PartialEq, Eq)]
enum SourceType {
    Linear,
    Concave,
    Convex,
    Switch,
}

#[derive(Debug, PartialEq, Eq)]
enum SourceDirection {
    Forward,
    Reverse,
}

#[derive(Debug, PartialEq, Eq)]
enum SourcePolarity {
    Unipolar,
    Bipolar,
}

#[derive(BinRead, Debug, PartialEq, Eq)]
enum ModularTransform {
    Linear,
    AbsoluteValue,
}

#[derive(Debug)]
struct Modulator {
    continuity: SourceType,
    polarity: SourcePolarity,
    direction: SourceDirection,
    index: SourceEnumeratorController,
}

fn parse_modulator(v: u16) -> Modulator {
    let continuity = match v >> 10 {
        0 => SourceType::Linear,
        1 => SourceType::Concave,
        2 => SourceType::Convex,
        3 => SourceType::Switch,
        _ => panic!(),
    };
    let polarity = if (v & 0x200) == 0x200 {
        SourcePolarity::Bipolar
    } else {
        SourcePolarity::Unipolar
    };
    let direction = if (v & 0x100) == 0x100 {
        SourceDirection::Reverse
    } else {
        SourceDirection::Forward
    };
    let index = if (v & 0x80) == 0x80 {
        SourceEnumeratorController::MidiCC((v & 0x7f) as u8)
    } else {
        match v & 0x7f {
            0 => SourceEnumeratorController::NoController,
            2 => SourceEnumeratorController::NoteOnVelocity,
            3 => SourceEnumeratorController::NoteOnKeyNumber,
            10 => SourceEnumeratorController::PolyPressure,
            13 => SourceEnumeratorController::ChannelPressure,
            14 => SourceEnumeratorController::PitchWheel,
            16 => SourceEnumeratorController::PitchWheelSensitivity,
            127 => SourceEnumeratorController::Link,
            _ => panic!(),
        }
    };
    Modulator {
        continuity,
        polarity,
        direction,
        index,
    }
}

#[derive(BinRead, Debug)]
struct ModList {
    #[br(map = |x: u16| parse_modulator(x))]
    src_oper: Modulator,
    dest_oper: u16,
    amount: i16,
    #[br(map = |x: u16| parse_modulator(x))]
    amt_src_oper: Modulator,
    #[br(pad_size_to = 2)]
    trans_oper: ModularTransform,
}

#[derive(BinRead, Debug)]
struct Bag {
    gen_ndx: u16,
    mod_ndx: u16,
}

#[derive(BinRead, Debug)]
struct Version {
    major: u16,
    minor: u16,
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

const RIFF: [u8; 4] = [b'R', b'I', b'F', b'F'];
const LIST: [u8; 4] = [b'L', b'I', b'S', b'T'];
const INAM: [u8; 4] = [b'I', b'N', b'A', b'M'];
const ICOP: [u8; 4] = [b'I', b'C', b'O', b'P'];
const ICRD: [u8; 4] = [b'I', b'C', b'R', b'D'];
const IPRD: [u8; 4] = [b'I', b'P', b'R', b'D'];
const ISFT: [u8; 4] = [b'I', b'S', b'F', b'T'];
const ICMT: [u8; 4] = [b'I', b'C', b'M', b'T'];
const IENG: [u8; 4] = [b'I', b'E', b'N', b'G'];
const ISNG: [u8; 4] = [b'i', b's', b'n', b'g'];
const IROM: [u8; 4] = [b'i', b'r', b'o', b'm'];
const IVER: [u8; 4] = [b'i', b'v', b'e', b'r'];
const IFIL: [u8; 4] = [b'i', b'f', b'i', b'l'];
const SDTA: [u8; 4] = [b's', b'd', b't', b'a'];
const SHDR: [u8; 4] = [b's', b'h', b'd', b'r'];
const SMPL: [u8; 4] = [b's', b'm', b'p', b'l'];
const PHDR: [u8; 4] = [b'p', b'h', b'd', b'r'];
const INST: [u8; 4] = [b'i', b'n', b's', b't'];
const IGEN: [u8; 4] = [b'i', b'g', b'e', b'n'];
const PGEN: [u8; 4] = [b'p', b'g', b'e', b'n'];
const IMOD: [u8; 4] = [b'i', b'm', b'o', b'd'];
const PMOD: [u8; 4] = [b'p', b'm', b'o', b'd'];
const IBAG: [u8; 4] = [b'i', b'b', b'a', b'g'];
const PBAG: [u8; 4] = [b'p', b'b', b'a', b'g'];

fn parse_soundfont(chunk: riff::Chunk, file: &mut File) {
    let mut todo = VecDeque::new();
    todo.push_back((chunk, 1));
    let mut samples = vec![];
    let mut _sample_data = vec![];
    let mut presets = vec![];
    let mut instruments = vec![];
    let mut igens = vec![];
    let mut pgens = vec![];
    let mut imods = vec![];
    let mut pmods = vec![];
    let mut ibags = vec![];
    let mut pbags = vec![];
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
                    RIFF | LIST | SDTA => {
                        for child in c.iter(file) {
                            todo.push_back((child, indent + 1));
                        }
                    }
                    IFIL | IVER => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        if let Ok(version) = reader.read_ne::<Version>() {
                            debug!(
                                "{chr:>indent$}Version: {}.{}",
                                version.major,
                                version.minor,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                        }
                    }
                    INAM | ISFT | IENG | ICOP | ISNG | IROM | ICRD | IPRD | ICMT => {
                        let data = c.read_contents(file).unwrap();
                        let name = String::from_utf8(data).unwrap();
                        debug!(
                            "{chr:>indent$}Name: {}",
                            name,
                            indent = 2 * (indent + 1),
                            chr = ' '
                        );
                    }
                    SMPL => {
                        _sample_data = c.read_contents(file).unwrap();
                        debug!(
                            "{chr:>indent$}Samples: {}",
                            c.len() / 2,
                            indent = 2 * (indent + 1),
                            chr = ' '
                        );
                    }
                    SHDR => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(sample) = reader.read_ne::<Sample>() {
                            if !sample.name.starts_with("EOS") {
                                debug!(
                                    "{chr:>indent$}Sample: {}",
                                    sample.name,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                            samples.push(sample);
                        }
                    }
                    PHDR => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(preset) = reader.read_ne::<Preset>() {
                            if !preset.name.starts_with("EOP") {
                                debug!(
                                    "{chr:>indent$}Preset: {}",
                                    preset.name,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                            presets.push(preset);
                        }
                    }
                    INST => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(instrument) = reader.read_ne::<Instrument>() {
                            if !instrument.name.starts_with("EOI") {
                                debug!(
                                    "{chr:>indent$}Instrument: {}",
                                    instrument.name,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                            instruments.push(instrument);
                        }
                    }
                    IGEN => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(generator) = reader.read_ne::<Generator>() {
                            if generator.oper != GeneratorType::StartAddrsOffset
                                && generator.oper != GeneratorType::EndAddrsOffset
                            {
                                debug!(
                                    "{chr:>indent$}Instrument Generator: {:?}, {}",
                                    generator.oper,
                                    generator.amount,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                            igens.push(generator);
                        }
                    }
                    PGEN => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(generator) = reader.read_ne::<Generator>() {
                            if generator.oper != GeneratorType::StartAddrsOffset
                                && generator.oper != GeneratorType::EndAddrsOffset
                            {
                                debug!(
                                    "{chr:>indent$}Preset Generator: {:?}, {}",
                                    generator.oper,
                                    generator.amount,
                                    indent = 2 * (indent + 1),
                                    chr = ' '
                                );
                            }
                            pgens.push(generator);
                        }
                    }
                    IMOD => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(mod_list) = reader.read_ne::<ModList>() {
                            debug!(
                                "{chr:>indent$}Instrument ModList: {:?}",
                                mod_list,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            imods.push(mod_list);
                        }
                    }
                    PMOD => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(mod_list) = reader.read_ne::<ModList>() {
                            debug!(
                                "{chr:>indent$}Preset ModList: {:?}",
                                mod_list,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            pmods.push(mod_list);
                        }
                    }
                    IBAG => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(bag) = reader.read_ne::<Bag>() {
                            debug!(
                                "{chr:>indent$}Instrument Bag: {:?}",
                                bag,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            ibags.push(bag);
                        }
                    }
                    PBAG => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(bag) = reader.read_ne::<Bag>() {
                            debug!(
                                "{chr:>indent$}Preset Bag: {:?}",
                                bag,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            pbags.push(bag);
                        }
                    }
                    _ => {}
                }
            }
            None => break,
        }
    }

    info!("Presets:");
    for ix in 0..presets.len() - 1 {
        let is_last = ix == presets.len() - 1;
        let preset = &presets[ix];
        info!("  Name: {}", preset.name);
        info!("  Pos: {}", preset.preset);
        info!("  Bank: {}", preset.bank);
        let bag_start = preset.bag_index as usize;
        let bag_end = if is_last {
            pbags.len()
        } else {
            let next_preset = &presets[ix + 1];
            next_preset.bag_index as usize
        };
	let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            info!("  Zone {}:", zone);
	    zone = zone + 1;
            let is_last = ix == pbags.len() - 1;
            let bag = &pbags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                pgens.len()
            } else {
		let next_bag = &pbags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            info!("    Generators:");
            for gen_ix in gen_start..gen_end {
                info!("      {:?}", pgens[gen_ix]);
            }
            let mod_start = bag.mod_ndx as usize;
            let mod_end = if is_last {
                pmods.len()
            } else {
		let next_bag = &pbags[bag_ix + 1];
                next_bag.mod_ndx as usize
            };
            info!("    Modulators:");
            for mod_ix in mod_start..mod_end {
                info!("      {:?}", pmods[mod_ix]);
            }
        }
	info!("");
    }
    info!("Instruments:");
    for ix in 0..instruments.len() - 1 {
        let is_last = ix == instruments.len() - 1;
        let instrument = &instruments[ix];
        info!("  Name: {}", instrument.name);
        let bag_start = instrument.bag_index as usize;
        let bag_end = if is_last {
            ibags.len()
        } else {
            let next_instrument = &instruments[ix + 1];
            next_instrument.bag_index as usize
        };
	let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            info!("  Zone {}:", zone);
	    zone = zone + 1;
            let is_last = ix == ibags.len() - 1;
            let bag = &ibags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                igens.len()
            } else {
		let next_bag = &ibags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            info!("    Generators:");
            for gen_ix in gen_start..gen_end {
                info!("      {:?}", igens[gen_ix]);
            }
            let mod_start = bag.mod_ndx as usize;
            let mod_end = if is_last {
                imods.len()
            } else {
		let next_bag = &ibags[bag_ix + 1];
                next_bag.mod_ndx as usize
            };
            info!("    Modulators:");
            for mod_ix in mod_start..mod_end {
                info!("      {:?}", imods[mod_ix]);
            }
        }
	info!("");
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut file = File::open(Path::new(filename)).unwrap();

    let chunk = riff::Chunk::read(&mut file, 0).unwrap();
    parse_soundfont(chunk, &mut file);
}
