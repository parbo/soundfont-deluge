use binread::*;
use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use xmlwriter::*;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LoopMode {
    NoLoop,
    ContinuousLoop,
    ReleaseLoop,
}

#[derive(BinRead, Debug)]
struct GeneratorData {
    oper: u16,
    amount: [u8; 2],
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Generator {
    StartAddrsOffset(i16),
    EndAddrsOffset(i16),
    StartloopAddrsOffset(i16),
    EndloopAddrsOffset(i16),
    StartAddrsCoarseOffset(i16),
    ModLfoToPitch(i16),
    VibLfoToPitch(i16),
    ModEnvToPitch(i16),
    InitialFilterFc(i16),
    InitialFilterQ(i16),
    ModLfoToFilterFc(i16),
    ModEnvToFilterFc(i16),
    EndAddrsCoarseOffset(i16),
    ModLfoToVolume(i16),
    ChorusEffectsSend(i16),
    ReverbEffectsSend(i16),
    Pan(i16),
    DelayModLFO(i16),
    FreqModLFO(i16),
    DelayVibLFO(i16),
    FreqVibLFO(i16),
    DelayModEnv(i16),
    AttackModEnv(i16),
    HoldModEnv(i16),
    DecayModEnv(i16),
    SustainModEnv(i16),
    ReleaseModEnv(i16),
    KeynumToModEnvHold(i16),
    KeynumToModEnvDecay(i16),
    DelayVolEnv(i16),
    SustainVolEnv(i16),
    AttackVolEnv(i16),
    HoldVolEnv(i16),
    DecayVolEnv(i16),
    ReleaseVolEnv(i16),
    KeynumToVolEnvHold(i16),
    KeynumToVolEnvDecay(i16),
    Instrument(u16),
    KeyRange(u8, u8),
    VelRange(u8, u8),
    StartloopAddrsCoarseOffset(i16),
    Keynum(i16),
    Velocity(i16),
    InitialAttenuation(i16),
    EndloopAddrsCoarseOffset(i16),
    CoarseTune(i16),
    FineTune(i16),
    SampleID(u16),
    SampleModes(LoopMode),
    ScaleTuning(i16),
    ExclusiveClass(i16),
    OverridingRootKey(i16),
    EndOper,
    Unused,
}

fn parse_generator(v: u16, a: [u8; 2]) -> Generator {
    match v {
        0 => Generator::StartAddrsOffset(i16::from_ne_bytes(a)),
        1 => Generator::EndAddrsOffset(i16::from_ne_bytes(a)),
        2 => Generator::StartloopAddrsOffset(i16::from_ne_bytes(a)),
        3 => Generator::EndloopAddrsOffset(i16::from_ne_bytes(a)),
        4 => Generator::StartAddrsCoarseOffset(i16::from_ne_bytes(a)),
        5 => Generator::ModLfoToPitch(i16::from_ne_bytes(a)),
        6 => Generator::VibLfoToPitch(i16::from_ne_bytes(a)),
        7 => Generator::ModEnvToPitch(i16::from_ne_bytes(a)),
        8 => Generator::InitialFilterFc(i16::from_ne_bytes(a)),
        9 => Generator::InitialFilterQ(i16::from_ne_bytes(a)),
        10 => Generator::ModLfoToFilterFc(i16::from_ne_bytes(a)),
        11 => Generator::ModEnvToFilterFc(i16::from_ne_bytes(a)),
        12 => Generator::EndAddrsCoarseOffset(i16::from_ne_bytes(a)),
        13 => Generator::ModLfoToVolume(i16::from_ne_bytes(a)),
        15 => Generator::ChorusEffectsSend(i16::from_ne_bytes(a)),
        16 => Generator::ReverbEffectsSend(i16::from_ne_bytes(a)),
        17 => Generator::Pan(i16::from_ne_bytes(a)),
        21 => Generator::DelayModLFO(i16::from_ne_bytes(a)),
        22 => Generator::FreqModLFO(i16::from_ne_bytes(a)),
        23 => Generator::DelayVibLFO(i16::from_ne_bytes(a)),
        24 => Generator::FreqVibLFO(i16::from_ne_bytes(a)),
        25 => Generator::DelayModEnv(i16::from_ne_bytes(a)),
        26 => Generator::AttackModEnv(i16::from_ne_bytes(a)),
        27 => Generator::HoldModEnv(i16::from_ne_bytes(a)),
        28 => Generator::DecayModEnv(i16::from_ne_bytes(a)),
        29 => Generator::SustainModEnv(i16::from_ne_bytes(a)),
        30 => Generator::ReleaseModEnv(i16::from_ne_bytes(a)),
        31 => Generator::KeynumToModEnvHold(i16::from_ne_bytes(a)),
        32 => Generator::KeynumToModEnvDecay(i16::from_ne_bytes(a)),
        33 => Generator::DelayVolEnv(i16::from_ne_bytes(a)),
        34 => Generator::AttackVolEnv(i16::from_ne_bytes(a)),
        35 => Generator::HoldVolEnv(i16::from_ne_bytes(a)),
        36 => Generator::DecayVolEnv(i16::from_ne_bytes(a)),
        37 => Generator::SustainVolEnv(i16::from_ne_bytes(a)),
        38 => Generator::ReleaseVolEnv(i16::from_ne_bytes(a)),
        39 => Generator::KeynumToVolEnvHold(i16::from_ne_bytes(a)),
        40 => Generator::KeynumToVolEnvDecay(i16::from_ne_bytes(a)),
        41 => Generator::Instrument(u16::from_ne_bytes(a)),
        43 => Generator::KeyRange(a[0], a[1]),
        44 => Generator::VelRange(a[0], a[1]),
        45 => Generator::StartloopAddrsCoarseOffset(i16::from_ne_bytes(a)),
        46 => Generator::Keynum(i16::from_ne_bytes(a)),
        47 => Generator::Velocity(i16::from_ne_bytes(a)),
        48 => Generator::InitialAttenuation(i16::from_ne_bytes(a)),
        50 => Generator::EndloopAddrsCoarseOffset(i16::from_ne_bytes(a)),
        51 => Generator::CoarseTune(i16::from_ne_bytes(a)),
        52 => Generator::FineTune(i16::from_ne_bytes(a)),
        53 => Generator::SampleID(u16::from_ne_bytes(a)),
        54 => Generator::SampleModes(match a[0] {
            1 => LoopMode::ContinuousLoop,
            3 => LoopMode::ReleaseLoop,
            _ => LoopMode::NoLoop,
        }),
        56 => Generator::ScaleTuning(i16::from_ne_bytes(a)),
        57 => Generator::ExclusiveClass(i16::from_ne_bytes(a)),
        58 => Generator::OverridingRootKey(i16::from_ne_bytes(a)),
        60 => Generator::EndOper,
        _x => {
            error!("Ununsed generator: {}", _x);
            Generator::Unused
        }
    }
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

#[derive(Debug)]
enum DestOper {
    Link(u16),
    Generator(Generator),
}

fn parse_dest_oper(v: u16) -> DestOper {
    if (v & 0x8000) == 0x8000 {
        DestOper::Link(v & 0x7ff)
    } else {
        DestOper::Generator(parse_generator(v, [0, 0]))
    }
}

#[derive(BinRead, Debug)]
struct ModList {
    #[br(map = |x: u16| parse_modulator(x))]
    src_oper: Modulator,
    #[br(map = |x: u16| parse_dest_oper(x))]
    dest_oper: DestOper,
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

struct SoundFont {
    samples: Vec<Sample>,
    sample_data: Vec<u8>,
    presets: Vec<Preset>,
    instruments: Vec<Instrument>,
    igens: Vec<Generator>,
    pgens: Vec<Generator>,
    imods: Vec<ModList>,
    pmods: Vec<ModList>,
    ibags: Vec<Bag>,
    pbags: Vec<Bag>,
}

fn parse_soundfont(chunk: riff::Chunk, file: &mut fs::File) -> SoundFont {
    let mut todo = VecDeque::new();
    todo.push_back((chunk, 1));
    let mut samples = vec![];
    let mut sample_data = vec![];
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
                        sample_data = c.read_contents(file).unwrap();
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
                        while let Ok(generator) = reader.read_ne::<GeneratorData>() {
                            debug!(
                                "{chr:>indent$}Instrument Generator: {:?}, {:?}",
                                generator.oper,
                                generator.amount,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            igens.push(parse_generator(generator.oper, generator.amount));
                        }
                    }
                    PGEN => {
                        let data = c.read_contents(file).unwrap();
                        let mut reader = Cursor::new(data);
                        while let Ok(generator) = reader.read_ne::<GeneratorData>() {
                            debug!(
                                "{chr:>indent$}Instrument Generator: {:?}, {:?}",
                                generator.oper,
                                generator.amount,
                                indent = 2 * (indent + 1),
                                chr = ' '
                            );
                            pgens.push(parse_generator(generator.oper, generator.amount));
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

    SoundFont {
        samples,
        sample_data,
        presets,
        instruments,
        igens,
        pgens,
        imods,
        pmods,
        ibags,
        pbags,
    }
}

impl SoundFont {
    fn dump(&self) {
        info!("Presets:");
        for ix in 0..self.presets.len() - 1 {
            self.dump_preset(ix);
        }
    }

    fn dump_preset(&self, ix: usize) {
        let is_last = ix == self.presets.len() - 1;
        let preset = &self.presets[ix];
        info!("  Name: {}", preset.name);
        info!("  Pos: {}", preset.preset);
        info!("  Bank: {}", preset.bank);
        let bag_start = preset.bag_index as usize;
        let bag_end = if is_last {
            self.pbags.len()
        } else {
            let next_preset = &self.presets[ix + 1];
            next_preset.bag_index as usize
        };
        let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            info!("  Preset zone {}:", zone);
            zone = zone + 1;
            let is_last = ix == self.pbags.len() - 1;
            let bag = &self.pbags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                self.pgens.len()
            } else {
                let next_bag = &self.pbags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            info!("    Generators:");
            for gen_ix in gen_start..gen_end {
                let gen = &self.pgens[gen_ix];
                match gen {
                    Generator::Instrument(index) => {
                        self.dump_instrument(*index as usize);
                    }
                    _ => {
                        info!("      {:?}", gen);
                    }
                }
            }
            let mod_start = bag.mod_ndx as usize;
            let mod_end = if is_last {
                self.pmods.len()
            } else {
                let next_bag = &self.pbags[bag_ix + 1];
                next_bag.mod_ndx as usize
            };
            info!("    Modulators:");
            for mod_ix in mod_start..mod_end {
                info!("      {:?}", self.pmods[mod_ix]);
            }
        }
        info!("");
    }

    fn dump_instrument(&self, ix: usize) {
        let is_last = ix == self.instruments.len() - 1;
        let instrument = &self.instruments[ix];
        info!("      Instrument: {}", instrument.name);
        let bag_start = instrument.bag_index as usize;
        let bag_end = if is_last {
            self.ibags.len()
        } else {
            let next_instrument = &self.instruments[ix + 1];
            next_instrument.bag_index as usize
        };
        let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            info!("        Instrument zone {}:", zone);
            zone = zone + 1;
            let is_last = ix == self.ibags.len() - 1;
            let bag = &self.ibags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                self.igens.len()
            } else {
                let next_bag = &self.ibags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            info!("          Generators:");
            for gen_ix in gen_start..gen_end {
                let gen = &self.igens[gen_ix];
                match gen {
                    Generator::SampleID(index) => {
                        info!("              {:?}", self.samples[*index as usize]);
                    }
                    _ => {
                        info!("            {:?}", gen);
                    }
                }
            }
            let mod_start = bag.mod_ndx as usize;
            let mod_end = if is_last {
                self.imods.len()
            } else {
                let next_bag = &self.ibags[bag_ix + 1];
                next_bag.mod_ndx as usize
            };
            info!("          Modulators:");
            for mod_ix in mod_start..mod_end {
                info!("             {:?}", self.imods[mod_ix]);
            }
        }
        info!("");
    }

    fn save_as_xml(&self, folder: &Path, sample_folder: &Path, ix: usize) {
        info!("Writing xml to {} for {}", folder.display(), ix);
        let mut w = XmlWriter::new(Options::default());
        w.write_declaration();
        w.start_element("sound");
        w.write_attribute("firmwareVersion", "3.1.3");
        w.write_attribute("earliestCompatibleFirmware", "3.1.0-beta");
        w.write_attribute("polyphonic", "poly");
        w.write_attribute("voicePriority", "1");
        w.write_attribute("mode", "subtractive");
        w.write_attribute("lpfMode", "24dB");
        w.write_attribute("modFxType", "none");
        let is_last = ix == self.presets.len() - 1;
        let preset = &self.presets[ix];
        let bag_start = preset.bag_index as usize;
        let bag_end = if is_last {
            self.pbags.len()
        } else {
            let next_preset = &self.presets[ix + 1];
            next_preset.bag_index as usize
        };
        let mut zones = vec![];
        let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            zone = zone + 1;
            let is_last = ix == self.pbags.len() - 1;
            let bag = &self.pbags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                self.pgens.len()
            } else {
                let next_bag = &self.pbags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            for gen_ix in gen_start..gen_end {
                let gen = &self.pgens[gen_ix];
                if let Generator::Instrument(index) = gen {
                    let mut gens = self.get_instrument_zones(*index as usize);
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
                    if let Some(Generator::KeyRange(low, _high)) =
                        SoundFont::get_zone_key_range(zone)
                    {
                        if osc.len() == 0 {
                            osc.push(zone_ix);
                            taken.insert(zone_ix);
                            found = true;
                        } else {
                            let prev_zone = osc.last().unwrap();
                            if let Some(Generator::KeyRange(_plow, phigh)) =
                                SoundFont::get_zone_key_range(&zones[*prev_zone])
                            {
                                if phigh + 1 == low {
                                    osc.push(zone_ix);
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
        let mut ix = 0;
        for osc in &oscs[0..2] {
            ix = ix + 1;
            w.start_element(&format!("osc{}", ix));
            w.write_attribute("type", "sample");
            w.write_attribute("loopMode", "0");
            w.write_attribute("reversed", "0");
            w.write_attribute("timeStretchEnable", "0");
            w.write_attribute("timeStretchAmount", "0");
            w.start_element("sampleRanges");
            for o in osc {
                w.start_element("sampleRange");
                if let Some(Generator::KeyRange(low, high)) =
                    SoundFont::get_zone_key_range(&zones[*o])
                {
                    w.write_attribute("rangeTopNote", &high.to_string());
                }
                if let Some(Generator::OverridingRootKey(root)) = SoundFont::get_zone_overriding_root_key(&zones[*o])
                {
		    // offset from middle c
		    w.write_attribute("transpose", &(60 - root).to_string())
		}
                if let Some(Generator::FineTune(cents)) = SoundFont::get_zone_fine_tune(&zones[*o])
                {
		    w.write_attribute("cents", &cents.to_string())
		}
                if let Some(Generator::SampleID(sample_id)) = SoundFont::get_zone_sample(&zones[*o])
                {
                    let sample = &self.samples[sample_id as usize];
                    let name = SoundFont::safe_name(&sample.name) + ".wav";
                    let file_path: Vec<String> = sample_folder
                        .join(name)
                        .components()
                        .map(|x| x.as_os_str().to_str().unwrap().into())
                        .collect();
                    w.write_attribute("fileName", &file_path.join("/"));
                    w.start_element("zone");
                    // TODO: take generator sample offsets into account
                    w.write_attribute("startSamplePos", "0");
                    w.write_attribute("endSamplePos", &(sample.end - sample.start).to_string());
                    w.end_element();
                }
                w.end_element();
            }
            w.end_element();
            w.end_element();
        }
        w.end_element();
        let xml = w.end_document();
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

    fn get_instrument_zones(&self, ix: usize) -> Vec<Vec<Generator>> {
        let mut zones = vec![];
        let is_last = ix == self.instruments.len() - 1;
        let instrument = &self.instruments[ix];
        let bag_start = instrument.bag_index as usize;
        let bag_end = if is_last {
            self.ibags.len()
        } else {
            let next_instrument = &self.instruments[ix + 1];
            next_instrument.bag_index as usize
        };
        let mut zone = 0;
        for bag_ix in bag_start..bag_end {
            zone = zone + 1;
            let is_last = ix == self.ibags.len() - 1;
            let bag = &self.ibags[bag_ix];
            let gen_start = bag.gen_ndx as usize;
            let gen_end = if is_last {
                self.igens.len()
            } else {
                let next_bag = &self.ibags[bag_ix + 1];
                next_bag.gen_ndx as usize
            };
            let mut zone = vec![];
            for gen_ix in gen_start..gen_end {
                let gen = &self.igens[gen_ix];
                zone.push(*gen);
            }
            zones.push(zone);
        }
        zones
    }

    fn safe_name(s: &str) -> String {
        s.chars()
            .map(|x| match x {
                '/' => '_',
                '"' => '_',
                _ => x,
            })
            .collect()
    }

    fn save_samples(&self, folder: &Path) -> std::io::Result<()> {
        info!("saving samples to {}", folder.display());
        fs::create_dir_all(folder)?;
        info!("created folder!");
        for sample in &self.samples {
            match sample.sample_type {
                1 | 2 | 4 => {
                    // TODO: maybe combine 2 and 4 to stereo sample?
                    info!("saving sample {}", sample.name);
                    let h = wav::Header::new(1, 1, sample.sample_rate, 16);
                    let name = SoundFont::safe_name(&sample.name) + ".wav";
                    let file_path = folder.join(name);
                    info!("file path: {}", file_path.display());
                    let mut out_file = fs::File::create(file_path)?;
                    info!("created file!");
                    let mut out = vec![];
                    let mut ix = sample.start * 2;
                    loop {
                        let low = self.sample_data[ix as usize] as i16;
                        let high = self.sample_data[(ix + 1) as usize] as i16;
                        out.push(high << 8 | low);
                        ix = ix + 2;
                        if ix >= 2 * sample.end {
                            break;
                        }
                    }
                    wav::write(h, wav::BitDepth::Sixteen(out), &mut out_file)?;
                }
                _ => {
                    warn!(
                        "Unsupported sample type: {}, name: {}",
                        sample.sample_type, sample.name
                    );
                }
            }
        }
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let folder = if args.len() > 2 { Some(&args[2]) } else { None };
    let xml_folder = if args.len() > 3 { Some(&args[3]) } else { None };

    let mut file = fs::File::open(Path::new(filename)).unwrap();

    let chunk = riff::Chunk::read(&mut file, 0).unwrap();
    let sf = parse_soundfont(chunk, &mut file);
    sf.dump_preset(2);
    sf.dump_preset(5);
    if let Some(folder) = folder {
        //        sf.save_samples(Path::new(folder)).unwrap();
        if let Some(xml_folder) = xml_folder {
            sf.save_as_xml(Path::new(xml_folder), Path::new(folder), 2);
            sf.save_as_xml(Path::new(xml_folder), Path::new(folder), 5);
        }
    }
}
