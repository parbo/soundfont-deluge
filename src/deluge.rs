use derive_builder::Builder;
use std::fs;
use std::io::{Read, Write};
use yaserde;
use yaserde::de::from_str;
use yaserde::ser::to_string_with_config;

// Value mappings
// Value        Level   LP Cutoff       HP Cutoff       LFO Rate                        Value   Pitch Mod Depth Attack          Decay           Release         Value
// 0    -               LO              LO              0.005 Hz        211.8 s         0       0 cents         0.7 ms          9 ms            0 ms            0
// 1    -68.0 dB        LO              LO              0.006 Hz        169.4 s         1       3 cents         0.9 ms          93 ms           44 ms           1
// 2    -56.0 dB        LO              LO              0.007 Hz        135.5 s         2       10 cents        1.0 ms          182 ms          91 ms           2
// 3    -48.9 dB        53 Hz           LO              0.009 Hz        108.4 s         3       20 cents        1.2 ms          276 ms          139 ms          3
// 4    -43.9 dB        53 Hz           LO              0.012 Hz        86.74 s         4       34 cents        1.4 ms          375 ms          189 ms          4
// 5    -40.0 dB        53 Hz           LO              0.014 Hz        69.39 s         5       52 cents        1.7 ms          478 ms          240 ms          5
// 6    -36.9 dB        53 Hz           LO              0.018 Hz        55.51 s         6       74 cents        2.0 ms          586 ms          294 ms          6
// 7    -34.2 dB        53 Hz           LO              0.023 Hz        44.41 s         7       99 cents        2.3 ms          699 ms          349 ms          7
// 8    -31.9 dB        53 Hz           19 Hz           0.028 Hz        35.53 s         8       129 cents       2.7 ms          0.82 s          407 ms          8
// 9    -29.8 dB        53 Hz           23 Hz           0.035 Hz        28.42 s         9       162 cents       3.2 ms          0.94 s          466 ms          9
// 10   -28.0 dB        61 Hz           29 Hz           0.044 Hz        22.74 s         10      198 cents       3.8 ms          1.07 s          527 ms          10
// 11   -26.3 dB        71 Hz           36 Hz           0.055 Hz        18.19 s         11      239 cents       4.5 ms          1.21 s          600 ms          11
// 12   -24.8 dB        83 Hz           46 Hz           0.069 Hz        14.55 s         12      283 cents       5.3 ms          1.36 s          675 ms          12
// 13   -23.4 dB        97 Hz           57 Hz           0.086 Hz        11.64 s         13      331 cents       6.3 ms          1.51 s          751 ms          13
// 14   -22.1 dB        113 Hz          71 Hz           0.107 Hz        9.31 s          14      383 cents       7.4 ms          1.67 s          830 ms          14
// 15   -20.9 dB        131 Hz          88 Hz           0.134 Hz        7.45 s          15      438 cents       8.8 ms          1.82 s          910 ms          15
// 16   -19.8 dB        153 Hz          109 Hz          0.168 Hz        5.96 s          16      498 cents       10 ms           1.99 s          0.99 s          16
// 17   -18.8 dB        178 Hz          135 Hz          0.210 Hz        4.77 s          17      561 cents       12 ms           2.16 s          1.08 s          17
// 18   -17.8 dB        208 Hz          168 Hz          0.262 Hz        3.81 s          18      627 cents       14 ms           2.33 s          1.17 s          18
// 19   -16.8 dB        242 Hz          208 Hz          0.328 Hz        3.05 s          19      698 cents       17 ms           2.52 s          1.26 s          19
// 20   -15.9 dB        281 Hz          258 Hz          0.410 Hz        2.44 s          20      772 cents       20 ms           2.71 s          1.36 s          20
// 21   -15.1 dB        328 Hz          320 Hz          0.512 Hz        1.95 s          21      850 cents       24 ms           2.91 s          1.46 s          21
// 22   -14.3 dB        381 Hz          396 Hz          0.640 Hz        1.56 s          22      932 cents       28 ms           3.12 s          1.57 s          22
// 23   -13.5 dB        444 Hz          491 Hz          0.800 Hz        1.25 s          23      1,018 cents     33 ms           3.35 s          1.69 s          23
// 24   -12.8 dB        516 Hz          607 Hz          1.00 Hz         1.00 s          24      1,107 cents     39 ms           3.58 s          1.81 s          24
// 25   -12.1 dB        600 Hz          750 Hz          1.25 Hz         800 ms          25      1,200 cents     46 ms           3.84 s          1.94 s          25
// 26   -11.4 dB        698 Hz          928 Hz          1.56 Hz         640 ms          26      1,298 cents     54 ms           4.11 s          2.08 s          26
// 27   -10.7 dB        812 Hz          1.1 kHz         1.95 Hz         512 ms          27      1,400 cents     64 ms           4.40 s          2.22 s          27
// 28   -10.1 dB        945 Hz          1.4 kHz         2.44 Hz         410 ms          28      1,506 cents     76 ms           4.72 s          2.38 s          28
// 29   -9.5 dB         1.1 kHz         1.7 kHz         3.05 Hz         328 ms          29      1,615 cents     90 ms           5.07 s          2.56 s          29
// 30   -8.9 dB         1.3 kHz         2.2 kHz         3.81 Hz         262 ms          30      1,729 cents     106 ms          5.44 s          2.75 s          30
// 31   -8.3 dB         1.5 kHz         2.7 kHz         4.77 Hz         210 ms          31      1,846 cents     125 ms          5.85 s          2.95 s          31
// 32   -7.8 dB         1.7 kHz         3.3 kHz         5.96 Hz         168 ms          32      1,967 cents     148 ms          6.31 s          3.18 s          32
// 33   -7.2 dB         2.0 kHz         4.0 kHz         7.45 Hz         134 ms          33      2,092 cents     174 ms          6.81 s          3.43 s          33
// 34   -6.7 dB         2.3 kHz         5.0 kHz         9 Hz            107 ms          34      2,220 cents     206 ms          7.37 s          3.71 s          34
// 35   -6.2 dB         2.7 kHz         6.1 kHz         12 Hz           86 ms           35      2,353 cents     243 ms          8.00 s          4.01 s          35
// 36   -5.7 dB         3.1 kHz         7.6 kHz         15 Hz           69 ms           36      2,489 cents     287 ms          8.70 s          4.36 s          36
// 37   -5.3 dB         3.7 kHz         9.3 kHz         18 Hz           55 ms           37      2,629 cents     339 ms          9.50 s          4.75 s          37
// 38   -4.8 dB         4.3 kHz         11.5 kHz        23 Hz           44 ms           38      2,773 cents     400 ms          10.4 s          5.18 s          38
// 39   -4.3 dB         4.9 kHz         14.1 kHz        28 Hz           35 ms           39      2,920 cents     472 ms          11.4 s          5.68 s          39
// 40   -3.9 dB         5.7 kHz         17.3 kHz        36 Hz           28 ms           40      3,072 cents     558 ms          12.6 s          6.24 s          40
// 41   -3.5 dB         6.7 kHz         18.0 kHz        44 Hz           23 ms           41      3,227 cents     658 ms          14.0 s          6.89 s          41
// 42   -3.0 dB         7.7 kHz         HI              56 Hz           18 ms           42      3,386 cents     777 ms          15.6 s          7.63 s          42
// 43   -2.6 dB         9.0 kHz         HI              69 Hz           14 ms           43      3,549 cents     918 ms          17.5 s          8.50 s          43
// 44   -2.2 dB         10.5 kHz        HI              87 Hz           12 ms           44      3,716 cents     1.08 s          19.8 s          9.50 s          44
// 45   -1.8 dB         12.1 kHz        HI              108 Hz          9 ms            45      3,886 cents     1.28 s          22.4 s          10.7 s          45
// 46   -1.5 dB         14.1 kHz        HI              136 Hz          7 ms            46      4,060 cents     1.51 s          25.6 s          12.1 s          46
// 47   -1.1 dB         16.4 kHz        HI              169 Hz          6 ms            47      4,238 cents     1.78 s          29.4 s          13.7 s          47
// 48   -0.7 dB         19.0 kHz        HI              212 Hz          5 ms            48      4,420 cents     2.11 s          34.0 s          15.6 s          48
// 49   -0.4 dB         22.1 kHz        HI              265 Hz          4 ms            49      4,606 cents     2.49 s          39.5 s          17.9 s          49
// 50   0.0 dB          HI              HI              331 Hz          3 ms            50      4,796 cents     2.94 s          46.4 s          20.7 s          50

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Value(pub u32);

impl Value {
    pub fn to_deluge_val(&self) -> i32 {
        let iv = self.0 as i32;
        let ivf = iv as f32;
        let ratio = ivf / i32::MAX as f32;
        (ratio * 25.0 + 25.0).round() as i32
    }

    pub fn from_deluge_val(v: i32) -> Value {
        let ratio = (v - 25) as f32 / 25.0;
        let iv = (ratio * i32::MAX as f32).round() as i32;
        Value(iv as u32)
    }
}

impl yaserde::YaSerialize for Value {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let s = format!("0x{:08X}", self.0);
        let _ret = writer.write(xml::writer::XmlEvent::characters(&s));
        Ok(())
    }
    fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
    ) -> Result<
        (
            Vec<xml::attribute::OwnedAttribute>,
            xml::namespace::Namespace,
        ),
        String,
    > {
        Ok((attributes, namespace))
    }
}

impl yaserde::YaDeserialize for Value {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        loop {
            match reader.next_event()? {
                xml::reader::XmlEvent::StartElement { .. } => {}
                xml::reader::XmlEvent::Characters(ref text_content) => {
                    return u32::from_str_radix(&text_content[2..], 16)
                        .map(Value)
                        .map_err(|e| e.to_string());
                }
                _ => {
                    break;
                }
            }
        }
        Err("Unable to parse Value".to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum OscType {
    #[yaserde(rename = "analogSaw")]
    AnalogSaw,
    #[yaserde(rename = "analogSquare")]
    AnalogSquare,
    #[yaserde(rename = "inLeft")]
    InLeft,
    #[yaserde(rename = "inRight")]
    InRight,
    #[yaserde(rename = "sample")]
    Sample,
    #[yaserde(rename = "saw")]
    Saw,
    #[yaserde(rename = "sine")]
    Sine,
    #[yaserde(rename = "square")]
    Square,
    #[yaserde(rename = "triangle")]
    Triangle,
}

impl Default for OscType {
    fn default() -> OscType {
        OscType::Sine
    }
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Zone {
    #[yaserde(attribute, rename = "startSamplePos")]
    start_sample_pos: u32,
    #[yaserde(attribute, rename = "endSamplePos")]
    end_sample_pos: u32,
    #[yaserde(attribute, rename = "startLoopPos")]
    start_loop_pos: Option<u32>,
    #[yaserde(attribute, rename = "endLoopPos")]
    end_loop_pos: Option<u32>,
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct SampleRange {
    #[yaserde(attribute, rename = "rangeTopNote")]
    range_top_note: Option<i32>,
    #[yaserde(attribute)]
    transpose: Option<i32>,
    #[yaserde(attribute)]
    cents: Option<i32>,
    #[yaserde(attribute, rename = "fileName")]
    file_name: Option<String>,
    zone: Zone,
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct SampleRanges {
    #[yaserde(rename = "sampleRange")]
    sample_range: Vec<SampleRange>,
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Osc {
    #[yaserde(attribute, rename = "type")]
    osc_type: OscType,
    #[yaserde(attribute)]
    transpose: Option<i32>,
    #[yaserde(attribute)]
    cents: Option<i32>,
    // TODO: make separate datatypes for each oscillator type instead of having Option
    // Regular wave oscillators
    #[yaserde(attribute, rename = "retrigPhase")]
    retrig_phase: Option<i32>,
    // Sample oscillator
    #[yaserde(attribute, rename = "fileName")]
    file_name: Option<String>,
    #[yaserde(attribute, rename = "loopMode")]
    loop_mode: Option<i32>,
    #[yaserde(attribute, rename = "reversed")]
    reversed: Option<i32>,
    #[yaserde(attribute, rename = "timeStretchEnable")]
    time_stretch_enable: Option<i32>,
    #[yaserde(attribute, rename = "timeStretchAmount")]
    time_stretch_amount: Option<i32>,
    #[yaserde(rename = "sampleRanges")]
    sample_ranges: Option<SampleRanges>,
    zone: Option<Zone>,
}

impl Default for Osc {
    fn default() -> Osc {
        Osc {
            osc_type: OscType::Square,
            transpose: Some(0),
            cents: Some(0),
            retrig_phase: Some(-1),
            file_name: None,
            loop_mode: None,
            reversed: None,
            time_stretch_enable: None,
            time_stretch_amount: None,
            sample_ranges: None,
            zone: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum LfoType {
    #[yaserde(rename = "saw")]
    Saw,
    #[yaserde(rename = "sine")]
    Sine,
    #[yaserde(rename = "square")]
    Square,
    #[yaserde(rename = "triangle")]
    Triangle,
}

impl Default for LfoType {
    fn default() -> LfoType {
        LfoType::Sine
    }
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Lfo {
    #[yaserde(attribute, rename = "type")]
    lfo_type: LfoType,
    #[yaserde(attribute, rename = "syncLevel")]
    sync_level: Option<i32>,
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum Mode {
    #[yaserde(rename = "ringmod")]
    Ringmod,
    #[yaserde(rename = "subtractive")]
    Subtractive,
    #[yaserde(rename = "fm")]
    Fm,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Subtractive
    }
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Unison {
    #[yaserde(attribute)]
    num: i32,
    #[yaserde(attribute)]
    detune: i32,
}

impl Default for Unison {
    fn default() -> Unison {
        Unison { num: 1, detune: 8 }
    }
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Delay {
    #[yaserde(attribute, rename = "pingPong")]
    ping_pong: i32,
    #[yaserde(attribute)]
    analog: i32,
    #[yaserde(attribute, rename = "syncLevel")]
    sync_level: i32,
}

impl Default for Delay {
    fn default() -> Delay {
        Delay {
            ping_pong: 1,
            analog: 0,
            sync_level: 7,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LpfMode {
    Mode24dB,
    Mode24dBDrive,
    Mode12dB,
}

// Need custom one for this since identifiers can't start with numbers
impl yaserde::YaSerialize for LpfMode {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        match *self {
            LpfMode::Mode24dB => writer
                .write(xml::writer::XmlEvent::characters("24dB"))
                .map_err(|e| e.to_string()),
            LpfMode::Mode24dBDrive => writer
                .write(xml::writer::XmlEvent::characters("24dBDrive"))
                .map_err(|e| e.to_string()),
            LpfMode::Mode12dB => writer
                .write(xml::writer::XmlEvent::characters("12dB"))
                .map_err(|e| e.to_string()),
        }
    }
    fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
    ) -> Result<
        (
            Vec<xml::attribute::OwnedAttribute>,
            xml::namespace::Namespace,
        ),
        String,
    > {
        Ok((attributes, namespace))
    }
}

impl yaserde::YaDeserialize for LpfMode {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::Characters(s) = reader.peek()?.to_owned() {
            match s.as_str() {
                "24dB" => Ok(LpfMode::Mode24dB),
                "24dBDrive" => Ok(LpfMode::Mode24dBDrive),
                "12dB" => Ok(LpfMode::Mode12dB),
                other => Err(other.to_string()),
            }
        } else {
            Err("Characters missing".to_string())
        }
    }
}

impl Default for LpfMode {
    fn default() -> LpfMode {
        LpfMode::Mode24dB
    }
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum ModFxType {
    #[yaserde(rename = "none")]
    None,
    #[yaserde(rename = "chorus")]
    Chorus,
    #[yaserde(rename = "flanger")]
    Flanger,
    #[yaserde(rename = "phaser")]
    Phaser,
}

impl Default for ModFxType {
    fn default() -> ModFxType {
        ModFxType::None
    }
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Envelope {
    #[yaserde(attribute)]
    attack: Value,
    #[yaserde(attribute)]
    decay: Value,
    #[yaserde(attribute)]
    sustain: Value,
    #[yaserde(attribute)]
    release: Value,
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum Source {
    #[yaserde(rename = "aftertouch")]
    Aftertouch,
    #[yaserde(rename = "compressor")]
    Compressor,
    #[yaserde(rename = "envelope1")]
    Envelope1,
    #[yaserde(rename = "envelope2")]
    Envelope2,
    #[yaserde(rename = "lfo1")]
    Lfo1,
    #[yaserde(rename = "lfo2")]
    Lfo2,
    #[yaserde(rename = "note")]
    Note,
    #[yaserde(rename = "velocity")]
    Velocity,
    #[yaserde(rename = "random")]
    Random,
}

impl Default for Source {
    fn default() -> Source {
        Source::Velocity
    }
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum Destination {
    #[yaserde(rename = "arpRate")]
    ArpRate,
    #[yaserde(rename = "bass")]
    Bass,
    #[yaserde(rename = "bassFreq")]
    BassFreq,
    #[yaserde(rename = "bitcrushAmount")]
    BitcrushAmount,
    #[yaserde(rename = "carrier1Feedback")]
    Carrier1Feedback,
    #[yaserde(rename = "carrier2Feedback")]
    Carrier2Feedback,
    #[yaserde(rename = "delayFeedback")]
    DelayFeedback,
    #[yaserde(rename = "delayRate")]
    DelayRate,
    #[yaserde(rename = "env1Attack")]
    Env1Attack,
    #[yaserde(rename = "env1Decay")]
    Env1Decay,
    #[yaserde(rename = "env1Release")]
    Env1Release,
    #[yaserde(rename = "env1Sustain")]
    Env1Sustain,
    #[yaserde(rename = "env2Attack")]
    Env2Attack,
    #[yaserde(rename = "env2Decay")]
    Env2Decay,
    #[yaserde(rename = "env2Release")]
    Env2Release,
    #[yaserde(rename = "env2Sustain")]
    Env2Sustain,
    #[yaserde(rename = "hpfFreuency")]
    HpfFrequency,
    #[yaserde(rename = "hpfResonance")]
    HpfResonance,
    #[yaserde(rename = "lfo1Rate")]
    Lfo1Rate,
    #[yaserde(rename = "lfo2Rate")]
    Lfo2Rate,
    #[yaserde(rename = "lpfFrequency")]
    LpfFrequency,
    #[yaserde(rename = "lpfResonance")]
    LpfResonance,
    #[yaserde(rename = "modFXDepth")]
    ModFxDepth,
    #[yaserde(rename = "modFXFeedback")]
    ModFxFeedback,
    #[yaserde(rename = "modFXRate")]
    ModFxRate,
    #[yaserde(rename = "modulator1Feedback")]
    Modulator1Feedback,
    #[yaserde(rename = "modulator1Pitch")]
    Modulator1Pitch,
    #[yaserde(rename = "modulator1Volume")]
    Modulator1Volume,
    #[yaserde(rename = "modulator2Feedback")]
    Modulator2Feedback,
    #[yaserde(rename = "modulator2Pitch")]
    Modulator2Pitch,
    #[yaserde(rename = "modulator2Volume")]
    Modulator2Volume,
    #[yaserde(rename = "noiseVolume")]
    NoiseVolume,
    #[yaserde(rename = "oscAPhaseWidth")]
    Osc1PhaseWidth,
    #[yaserde(rename = "oscAPitch")]
    Osc1Pitch,
    #[yaserde(rename = "oscAVolume")]
    Osc1Volume,
    #[yaserde(rename = "oscBPhaseWidth")]
    Osc2PhaseWidth,
    #[yaserde(rename = "oscBPitch")]
    Osc2Pitch,
    #[yaserde(rename = "oscBVolume")]
    Osc2Volume,
    #[yaserde(rename = "pan")]
    Pan,
    #[yaserde(rename = "pitch")]
    Pitch,
    #[yaserde(rename = "portamento")]
    Portamento,
    #[yaserde(rename = "range")]
    Range,
    #[yaserde(rename = "reverbAmount")]
    ReverbAmount,
    #[yaserde(rename = "sampleRateReduction")]
    SampleRateReduction,
    #[yaserde(rename = "stutterRate")]
    StutterRate,
    #[yaserde(rename = "treble")]
    Treble,
    #[yaserde(rename = "trebleFreq")]
    TrebleFreq,
    #[yaserde(rename = "volume")]
    Volume,
    #[yaserde(rename = "volumePostFX")]
    VolumePostFx,
    #[yaserde(rename = "volumePostReverbSend")]
    VolumePostReverbSend,
}

impl Default for Destination {
    fn default() -> Destination {
        Destination::Volume
    }
}

#[derive(YaSerialize, Clone, Builder, YaDeserialize, Debug, Eq, PartialEq)]
pub struct PatchCable {
    #[yaserde(attribute)]
    source: Source,
    #[yaserde(attribute)]
    destination: Destination,
    #[yaserde(attribute)]
    amount: Value,
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Equalizer {
    #[yaserde(attribute)]
    bass: Value,
    #[yaserde(attribute)]
    treble: Value,
    #[yaserde(attribute)]
    bass_frequency: Value,
    #[yaserde(attribute)]
    treble_frequency: Value,
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Compressor {
    #[yaserde(attribute, rename = "syncLevel")]
    sync_level: i32,
    #[yaserde(attribute)]
    attack: i32,
    #[yaserde(attribute)]
    release: i32,
}

impl Default for Compressor {
    fn default() -> Compressor {
        Compressor {
            sync_level: 7,
            attack: 327244,
            release: 936,
        }
    }
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct PatchCables {
    #[yaserde(rename = "patchCable")]
    patch_cable: Vec<PatchCable>,
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct DefaultParams {
    #[yaserde(attribute, rename = "arpeggiatorGate")]
    arpeggiator_gate: Value,
    #[yaserde(attribute)]
    portamento: Value,
    #[yaserde(attribute, rename = "compressorShape")]
    compressor_shape: Value,
    #[yaserde(attribute, rename = "oscAVolume")]
    osc1_volume: Value,
    #[yaserde(attribute, rename = "oscAPulseWidth")]
    osc1_pulse_width: Value,
    #[yaserde(attribute, rename = "oscBVolume")]
    osc2_volume: Value,
    #[yaserde(attribute, rename = "oscBPulseWidth")]
    osc2_pulse_width: Value,
    #[yaserde(attribute, rename = "noiseVolume")]
    noise_volume: Value,
    #[yaserde(attribute)]
    volume: Value,
    #[yaserde(attribute)]
    pan: Value,
    #[yaserde(attribute, rename = "lpfFrequency")]
    lpf_frequency: Value,
    #[yaserde(attribute, rename = "lpfResonance")]
    lpf_resonance: Value,
    #[yaserde(attribute, rename = "hpfFrequency")]
    hpf_frequency: Value,
    #[yaserde(attribute, rename = "hpfResonance")]
    hpf_resonance: Value,
    envelope1: Envelope,
    envelope2: Envelope,
    #[yaserde(attribute, rename = "lfo1Rate")]
    lfo1_rate: Value,
    #[yaserde(attribute, rename = "lfo2Rate")]
    lfo2_rate: Value,
    #[yaserde(attribute, rename = "modulator1Amount")]
    modulator1_amount: Value,
    #[yaserde(attribute, renanme = "modulator1Feedback")]
    modulator1_feedback: Value,
    #[yaserde(attribute, rename = "modulator2Amount")]
    modulator2_amount: Value,
    #[yaserde(attribute, rename = "modulator2Feedback")]
    modulator2_feedback: Value,
    #[yaserde(attribute, rename = "carrier1Feedback")]
    carrier1_feedback: Value,
    #[yaserde(attribute, rename = "carrier2Feedback")]
    carrier2_feedback: Value,
    #[yaserde(attribute, rename = "modFXRate")]
    mod_fx_rate: Value,
    #[yaserde(attribute, rename = "modFXDepth")]
    mod_fx_depth: Value,
    #[yaserde(attribute, rename = "delayRate")]
    delay_rate: Value,
    #[yaserde(attribute, rename = "delayFeedback")]
    delay_feedback: Value,
    #[yaserde(attribute, rename = "reverbAmount")]
    reverb_amount: Value,
    #[yaserde(attribute, rename = "arpeggiatorRate")]
    arpeggiator_rate: Value,
    #[yaserde(rename = "patchCables")]
    patch_cables: PatchCables,
    #[yaserde(attribute, rename = "stutterRate")]
    stutter_rate: Value,
    #[yaserde(attribute, rename = "sampleRateReduction")]
    sample_rate_reduction: Value,
    #[yaserde(attribute, rename = "bitCrush")]
    bitcrush: Value,
    equalizer: Equalizer,
    #[yaserde(attribute, rename = "modFXOffset")]
    mod_fx_offset: Value,
    #[yaserde(attribute, rename = "modFXFeedback")]
    mod_fx_feedback: Value,
}

// Values from saved init patch (Init.xml)
impl Default for DefaultParams {
    fn default() -> DefaultParams {
        let volume = PatchCable {
            source: Source::Velocity,
            destination: Destination::Volume,
            amount: Value(0x3FFFFFE8),
        };
        DefaultParams {
            arpeggiator_gate: Value(0x00000000),
            portamento: Value(0x80000000),
            compressor_shape: Value(0xDC28F5B2),
            osc1_volume: Value(0x7FFFFFFF),
            osc1_pulse_width: Value(0x00000000),
            osc2_volume: Value(0x80000000),
            osc2_pulse_width: Value(0x00000000),
            noise_volume: Value(0x80000000),
            volume: Value(0x4CCCCCA8),
            pan: Value(0x00000000),
            lpf_frequency: Value(0x7FFFFFFF),
            lpf_resonance: Value(0x80000000),
            hpf_frequency: Value(0x80000000),
            hpf_resonance: Value(0x80000000),
            envelope1: Envelope {
                attack: Value(0x80000000),
                decay: Value(0xE6666654),
                sustain: Value(0x7FFFFFFF),
                release: Value(0x80000000),
            },
            envelope2: Envelope {
                attack: Value(0xE6666654),
                decay: Value(0xE6666654),
                sustain: Value(0xFFFFFFE9),
                release: Value(0xE6666654),
            },
            lfo1_rate: Value(0x1999997E),
            lfo2_rate: Value(0x00000000),
            modulator1_amount: Value(0x80000000),
            modulator1_feedback: Value(0x80000000),
            modulator2_amount: Value(0x80000000),
            modulator2_feedback: Value(0x80000000),
            carrier1_feedback: Value(0x80000000),
            carrier2_feedback: Value(0x80000000),
            mod_fx_rate: Value(0x80000000),
            mod_fx_depth: Value(0x80000000),
            delay_rate: Value(0x00000000),
            delay_feedback: Value(0x80000000),
            reverb_amount: Value(0x80000000),
            arpeggiator_rate: Value(0x00000000),
            patch_cables: PatchCables {
                patch_cable: vec![volume],
            },
            stutter_rate: Value(0x00000000),
            sample_rate_reduction: Value(0x80000000),
            bitcrush: Value(0x80000000),
            equalizer: Equalizer::default(),
            mod_fx_offset: Value(0x00000000),
            mod_fx_feedback: Value(0x00000000),
        }
    }
}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct MidiKnob {}

#[derive(Default, Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[yaserde(rename_all = "camelCase")]
#[builder(default)]
pub struct MidiKnobs {
    midi_knob: Vec<MidiKnob>,
}

#[derive(YaSerialize, Clone, Builder, YaDeserialize, Debug, Eq, PartialEq)]
#[yaserde(rename_all = "camelCase")]
pub struct ModKnob {
    #[yaserde(attribute, rename = "controlsParam")]
    controls_param: Destination,
    #[yaserde(attribute, rename = "patchAmountFromSource")]
    patch_amount_from_source: Option<Source>,
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[yaserde(rename_all = "camelCase")]
#[builder(default)]
pub struct ModKnobs {
    mod_knob: Vec<ModKnob>,
}

impl Default for ModKnobs {
    fn default() -> ModKnobs {
        ModKnobs {
            mod_knob: vec![
                ModKnob {
                    controls_param: Destination::Pan,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::VolumePostFx,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::LpfResonance,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::LpfFrequency,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::Env1Release,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::Env1Attack,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::DelayFeedback,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::DelayRate,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::ReverbAmount,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::VolumePostReverbSend,
                    patch_amount_from_source: Some(Source::Compressor),
                },
                ModKnob {
                    controls_param: Destination::Pitch,
                    patch_amount_from_source: Some(Source::Lfo1),
                },
                ModKnob {
                    controls_param: Destination::Lfo1Rate,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::Portamento,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::StutterRate,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::BitcrushAmount,
                    patch_amount_from_source: None,
                },
                ModKnob {
                    controls_param: Destination::SampleRateReduction,
                    patch_amount_from_source: None,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, YaSerialize, YaDeserialize)]
pub enum ArpeggiatorMode {
    #[yaserde(rename = "off")]
    Off,
}

impl Default for ArpeggiatorMode {
    fn default() -> ArpeggiatorMode {
        ArpeggiatorMode::Off
    }
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Arpeggiator {
    #[yaserde(attribute)]
    mode: ArpeggiatorMode,
    #[yaserde(attribute, rename = "numOctaves")]
    num_octaves: i32,
    #[yaserde(attribute, rename = "syncLevel")]
    sync_level: i32,
}

impl Default for Arpeggiator {
    fn default() -> Arpeggiator {
        Arpeggiator {
            mode: ArpeggiatorMode::Off,
            num_octaves: 2,
            sync_level: 7,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Polyphony {
    Auto,
    Mono,
    Legato,
    Poly,
    Integer(u32),
}

impl Default for Polyphony {
    fn default() -> Polyphony {
        Polyphony::Poly
    }
}

impl yaserde::YaSerialize for Polyphony {
    fn serialize<W: Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        match *self {
            Polyphony::Auto => writer
                .write(xml::writer::XmlEvent::characters("auto"))
                .map_err(|e| e.to_string()),
            Polyphony::Mono => writer
                .write(xml::writer::XmlEvent::characters("mono"))
                .map_err(|e| e.to_string()),
            Polyphony::Legato => writer
                .write(xml::writer::XmlEvent::characters("legato"))
                .map_err(|e| e.to_string()),
            Polyphony::Poly => writer
                .write(xml::writer::XmlEvent::characters("poly"))
                .map_err(|e| e.to_string()),
            Polyphony::Integer(x) => writer
                .write(xml::writer::XmlEvent::characters(&x.to_string()))
                .map_err(|e| e.to_string()),
        }
    }
    fn serialize_attributes(
        &self,
        attributes: Vec<xml::attribute::OwnedAttribute>,
        namespace: xml::namespace::Namespace,
    ) -> Result<
        (
            Vec<xml::attribute::OwnedAttribute>,
            xml::namespace::Namespace,
        ),
        String,
    > {
        Ok((attributes, namespace))
    }
}

impl yaserde::YaDeserialize for Polyphony {
    fn deserialize<R: Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::Characters(s) = reader.peek()?.to_owned() {
            match s.as_str() {
                "auto" => Ok(Polyphony::Auto),
                "mono" => Ok(Polyphony::Mono),
                "legato" => Ok(Polyphony::Legato),
                "poly" => Ok(Polyphony::Poly),
                other => {
                    if let Ok(v) = other.parse::<u32>() {
                        Ok(Polyphony::Integer(v))
                    } else {
                        Err(other.to_string())
                    }
                }
            }
        } else {
            Err("Characters missing".to_string())
        }
    }
}

#[derive(Clone, Builder, YaSerialize, YaDeserialize, Debug, Eq, PartialEq)]
#[yaserde(rename = "sound")]
#[builder(default)]
pub struct Sound {
    #[yaserde(attribute,. rename = "firmwareVersion")]
    firmware_version: Option<String>,
    #[yaserde(attribute, rename = "earliestCompatibleFirmware")]
    earliest_compatible_firmware: Option<String>,
    osc1: Osc,
    osc2: Osc,
    #[yaserde(attribute)]
    polyphonic: Polyphony,
    #[yaserde(attribute, rename = "clippingAmount")]
    clipping_amount: Option<u32>,
    #[yaserde(attribute, rename = "voicePriority")]
    voice_priority: u32,
    lfo1: Lfo,
    lfo2: Lfo,
    #[yaserde(attribute)]
    mode: Mode,
    #[yaserde(attribute, rename = "lpfMode")]
    lpf_mode: Option<LpfMode>,
    unison: Unison,
    delay: Delay,
    compressor: Option<Compressor>,
    #[yaserde(attribute, rename = "modFXType")]
    mod_fx_type: ModFxType,
    #[yaserde(rename = "defaultParams")]
    default_params: DefaultParams,
    arpeggiator: Arpeggiator,
    #[yaserde(rename = "midiKnobs")]
    midi_knobs: Option<MidiKnobs>,
    #[yaserde(rename = "modKnobs")]
    mod_knobs: ModKnobs,
}

impl Default for Sound {
    fn default() -> Sound {
        Sound {
            firmware_version: Some("3.1.3".to_string()),
            earliest_compatible_firmware: Some("3.1.3-beta".to_string()),
            osc1: Osc::default(),
            osc2: Osc::default(),
            polyphonic: Polyphony::default(),
            clipping_amount: None,
            voice_priority: 1,
            lfo1: Lfo {
                lfo_type: LfoType::Triangle,
                sync_level: Some(0),
            },
            lfo2: Lfo {
                lfo_type: LfoType::Triangle,
                sync_level: None,
            },
            mode: Mode::default(),
            lpf_mode: Some(LpfMode::default()),
            unison: Unison::default(),
            delay: Delay::default(),
            compressor: Some(Compressor::default()),
            mod_fx_type: ModFxType::default(),
            default_params: DefaultParams::default(),
            arpeggiator: Arpeggiator::default(),
            midi_knobs: None,
            mod_knobs: ModKnobs::default(),
        }
    }
}

impl Sound {
    pub fn to_xml(&self) -> String {
        let yaserde_cfg = yaserde::ser::Config {
            perform_indent: false,
            ..Default::default()
        };
        // serialize
        let xml = to_string_with_config(self, &yaserde_cfg).unwrap();
        // reformat
        let mut out = String::new();
        let mut indent_level = -1i32;
        let mut in_string = false;
        let mut _prev = None;
        let mut chars = xml.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '>' => {
                    out.push(c);
                    if !in_string {
                        out.push('\n');
                        if indent_level > 0 {
                            out.push_str(&"\t".repeat(indent_level as usize));
                        }
                    }
                }
                '<' => {
                    if in_string {
                        out.push(c);
                    } else {
                        if chars.peek() == Some(&'/') {
                            if out.chars().last() == Some('\t') {
                                out.pop();
                            }
                            out.push(c);
                        } else {
                            out.push(c);
                            indent_level += 1;
                        }
                    }
                }
                '/' => {
                    if !in_string {
                        indent_level -= 1;
                    }
                    out.push(c);
                }
                '"' => {
                    in_string = !in_string;
                    out.push(c);
                }
                ' ' => {
                    if in_string {
                        out.push(c);
                    } else {
                        if chars.peek() == Some(&'/') {
                            out.push(c);
                        } else {
                            if indent_level > 0 {
                                out.push('\n');
                                out.push_str(&"\t".repeat(indent_level as usize));
                            } else {
                                out.push(c);
                            }
                        }
                    }
                }
                _ => {
                    out.push(c);
                }
            }
            _prev = Some(c);
        }
        out = out.replace("utf-8", "UTF-8");
        out
    }

    pub fn from_xml(file: &mut fs::File) -> Sound {
        let mut s = String::new();
        let _ = file.read_to_string(&mut s).unwrap();
        from_str(&s).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_type() {
        let s = "<type>saw</type>";
        let expected = OscType::Saw;
        let parsed: OscType = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_osc() {
        let s = "<osc1><type>saw</type><transpose>0</transpose><cents>0</cents><retrigPhase>-1</retrigPhase></osc1>";
        let expected = OscBuilder::default()
            .osc_type(OscType::Saw)
            .transpose(Some(0))
            .cents(Some(0))
            .retrig_phase(Some(-1))
            .build()
            .unwrap();
        let parsed: Osc = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_lfo_type() {
        let xml = "<type>sine</type>";
        let value = LfoType::Sine;
        let parsed: LfoType = from_str(&xml).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn test_lfo() {
        let s = "<lfo1><type>triangle</type><syncLevel>0</syncLevel></lfo1>";
        let expected = Lfo {
            lfo_type: LfoType::Triangle,
            sync_level: Some(0),
        };
        let parsed: Lfo = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_envelope() {
        let s = "<envelope1 attack=\"0x80000000\" decay=\"0xE6666654\" sustain=\"0x7FFFFFFF\" release=\"0x80000000\"></envelope1>";
        let expected = Envelope {
            attack: Value(0x80000000),
            decay: Value(0xE6666654),
            sustain: Value(0x7FFFFFFF),
            release: Value(0x80000000),
        };
        let parsed: Envelope = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_patch_cable() {
        let s = "<patchCable source=\"velocity\" destination=\"volume\" amount=\"0x3FFFFFE8\"></patchCable>";
        let expected = PatchCable {
            source: Source::Velocity,
            destination: Destination::Volume,
            amount: Value(0x3FFFFFE8),
        };
        let parsed: PatchCable = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_patch_cables() {
        let s = "<patchCables><patchCable source=\"velocity\" destination=\"volume\" amount=\"0x3FFFFFE8\"></patchCable></patchCables>";
        let expected = PatchCables {
            patch_cable: vec![PatchCable {
                source: Source::Velocity,
                destination: Destination::Volume,
                amount: Value(0x3FFFFFE8),
            }],
        };
        let parsed: PatchCables = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_build_sound() {
        let sound = SoundBuilder::default()
            .osc2(
                OscBuilder::default()
                    .osc_type(OscType::Saw)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        assert_eq!(sound.osc2.osc_type, OscType::Saw);
        println!("{:?}", sound);
        let s = sound.to_xml();
        for line in s.lines() {
            println!("{}", line);
        }
    }

    #[test]
    fn test_value() {
        assert_eq!(Value(0x80000000).to_deluge_val(), 0);
        assert_eq!(Value(0x00000000).to_deluge_val(), 25);
        assert_eq!(Value(0x7FFFFFFF).to_deluge_val(), 50);
        assert_eq!(Value::from_deluge_val(0), Value(0x80000000));
        assert_eq!(Value::from_deluge_val(25), Value(0x00000000));
        assert_eq!(Value::from_deluge_val(50), Value(0x7FFFFFFF));
    }
}
