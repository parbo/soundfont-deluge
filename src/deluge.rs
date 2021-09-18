use deluge_macros::serde_enum;
use derive_builder::Builder;
use quick_xml::de::from_str;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::io::Read;

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Value(u32);

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:08x}", self.0);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        u32::from_str_radix(&s[2..], 16)
            .map(Value)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum OscType {
    AnalogSaw,
    AnalogSquare,
    InLeft,
    InRight,
    Sample,
    Saw,
    Sine,
    Square,
    Triangle,
}

impl Default for OscType {
    fn default() -> OscType {
        OscType::Sine
    }
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    start_sample_pos: u32,
    end_sample_pos: u32,
    start_loop_pos: Option<u32>,
    end_loop_pos: Option<u32>,
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SampleRange {
    range_top_note: Option<i32>,
    transpose: Option<i32>,
    cents: Option<i32>,
    file_name: String,
    zone: Zone,
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct SampleRanges {
    sample_range: Vec<SampleRange>,
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Osc {
    #[serde(rename = "type", default)]
    osc_type: OscType,
    transpose: Option<i32>,
    cents: Option<i32>,
    // TODO: make separate datatypes for each oscillator type instead of having Option
    // Regular wave oscillators
    retrig_phase: Option<i32>,
    // Sample oscillator
    loop_mode: Option<i32>,
    reversed: Option<i32>,
    time_stretch_enable: Option<i32>,
    time_stretch_amount: Option<i32>,
    sample_ranges: Option<SampleRanges>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum LfoType {
    Saw,
    Sine,
    Square,
    Triangle,
}

impl Default for LfoType {
    fn default() -> LfoType {
        LfoType::Sine
    }
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Lfo {
    #[serde(rename = "type")]
    lfo_type: LfoType,
    sync_level: Option<i32>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum Mode {
    Ringmod,
    Subtractive,
    Fm,
}

impl Default for Mode {
    fn default() -> Mode {
        Mode::Subtractive
    }
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Unison {
    num: i32,
    detune: i32,
}

impl Default for Unison {
    fn default() -> Unison {
        Unison { num: 1, detune: 8 }
    }
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Delay {
    ping_pong: i32,
    analog: i32,
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
impl Serialize for LpfMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            LpfMode::Mode24dB => serializer.serialize_str("24dB"),
            LpfMode::Mode24dBDrive => serializer.serialize_str("24dBDrive"),
            LpfMode::Mode12dB => serializer.serialize_str("12dB"),
        }
    }
}

impl<'de> Deserialize<'de> for LpfMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "24dB" => Ok(LpfMode::Mode24dB),
            "24dBDrive" => Ok(LpfMode::Mode24dBDrive),
            "12dB" => Ok(LpfMode::Mode12dB),
            other => Err(serde::de::Error::custom(other.to_string())),
        }
    }
}

impl Default for LpfMode {
    fn default() -> LpfMode {
        LpfMode::Mode24dBDrive
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum ModFxType {
    None,
    Chorus,
    Flanger,
    Phaser,
}

impl Default for ModFxType {
    fn default() -> ModFxType {
        ModFxType::None
    }
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Envelope {
    attack: Value,
    decay: Value,
    sustain: Value,
    release: Value,
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum Source {
    Aftertouch,
    Compressor,
    Envelope1,
    Envelope2,
    Lfo1,
    Lfo2,
    Note,
    Velocity,
    Random,
}

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum Destination {
    ArpRate,
    Bass,
    BassFreq,
    BitcrushAmount,
    Carrier1Feedback,
    Carrier2Feedback,
    DelayFeedback,
    DelayRate,
    Env1Attack,
    Env1Decay,
    Env1Release,
    Env1Sustain,
    Env2Attack,
    Env2Decay,
    Env2Release,
    Env2Sustain,
    HpfFrequency,
    HpfResonance,
    Lfo1Rate,
    Lfo2Rate,
    LpfFrequency,
    LpfResonance,
    ModFxDepth,
    ModFxFeedback,
    ModFxRate,
    Modulator1Feedback,
    Modulator1Pitch,
    Modulator1Volume,
    Modulator2Feedback,
    Modulator2Pitch,
    Modulator2Volume,
    NoiseVolume,
    OscAPhaseWidth,
    OscAPitch,
    OscAVolume,
    OscBPhaseWidth,
    OscBPitch,
    OscBVolume,
    Pan,
    Pitch,
    Portamento,
    Range,
    ReverbAmount,
    SampleRateReduction,
    StutterRate,
    Treble,
    TrebleFreq,
    Volume,
    VolumePostFx,
    VolumePostReverbSend,
}

#[derive(Serialize, Clone, Builder, Deserialize, Debug, Eq, PartialEq)]
pub struct PatchCable {
    source: Source,
    destination: Destination,
    amount: Value,
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Equalizer {
    bass: Value,
    treble: Value,
    bass_frequency: Value,
    treble_frequency: Value,
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Compressor {
    sync_level: i32,
    attack: i32,
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

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct PatchCables {
    patch_cable: Vec<PatchCable>,
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct DefaultParams {
    arpeggiator_gate: Value,
    portamento: Value,
    compressor_shape: Value,
    osc_a_volume: Value,
    osc_a_pulse_width: Value,
    osc_b_volume: Value,
    osc_b_pulse_width: Value,
    noise_volume: Value,
    volume: Value,
    pan: Value,
    lpf_frequency: Value,
    lpf_resonance: Value,
    hpf_frequency: Value,
    hpf_resonance: Value,
    envelope_1: Envelope,
    envelope_2: Envelope,
    lfo_1_rate: Value,
    lfo_2_rate: Value,
    modulator_1_amount: Value,
    modulator_1_feedback: Value,
    modulator_2_amount: Value,
    modulator_2_feedback: Value,
    carrier_1_feedback: Value,
    carrier_2_feedback: Value,
    #[serde(rename = "modFXRate")]
    mod_fx_rate: Value,
    #[serde(rename = "modFXDepth")]
    mod_fx_depth: Value,
    delay_rate: Value,
    delay_feedback: Value,
    reverb_amount: Value,
    arpeggiator_rate: Value,
    patch_cables: PatchCables,
    stutter_rate: Value,
    sample_rate_reduction: Value,
    bit_crush: Value,
    equalizer: Equalizer,
    #[serde(rename = "modFXOffset")]
    mod_fx_offset: Value,
    #[serde(rename = "modFXFeedback")]
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
            osc_a_volume: Value(0x7FFFFFFF),
            osc_a_pulse_width: Value(0x00000000),
            osc_b_volume: Value(0x80000000),
            osc_b_pulse_width: Value(0x00000000),
            noise_volume: Value(0x80000000),
            volume: Value(0x4CCCCCA8),
            pan: Value(0x00000000),
            lpf_frequency: Value(0x7FFFFFFF),
            lpf_resonance: Value(0x80000000),
            hpf_frequency: Value(0x80000000),
            hpf_resonance: Value(0x80000000),
            envelope_1: Envelope {
                attack: Value(0x80000000),
                decay: Value(0xE6666654),
                sustain: Value(0x7FFFFFFF),
                release: Value(0x80000000),
            },
            envelope_2: Envelope {
                attack: Value(0xE6666654),
                decay: Value(0xE6666654),
                sustain: Value(0xFFFFFFE9),
                release: Value(0xE6666654),
            },
            lfo_1_rate: Value(0x1999997E),
            lfo_2_rate: Value(0x00000000),
            modulator_1_amount: Value(0x80000000),
            modulator_1_feedback: Value(0x80000000),
            modulator_2_amount: Value(0x80000000),
            modulator_2_feedback: Value(0x80000000),
            carrier_1_feedback: Value(0x80000000),
            carrier_2_feedback: Value(0x80000000),
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
            bit_crush: Value(0x80000000),
            equalizer: Equalizer::default(),
            mod_fx_offset: Value(0x00000000),
            mod_fx_feedback: Value(0x00000000),
        }
    }
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct MidiKnob {}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct MidiKnobs {
    #[serde(default)]
    midi_knob: Vec<MidiKnob>,
}

#[derive(Serialize, Clone, Builder, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModKnob {
    controls_param: Destination,
    patch_amount_from_source: Option<Source>,
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct ModKnobs {
    #[serde(default)]
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

#[derive(Debug, Clone, Eq, PartialEq, serde_enum)]
pub enum ArpeggiatorMode {
    Off,
}

impl Default for ArpeggiatorMode {
    fn default() -> ArpeggiatorMode {
        ArpeggiatorMode::Off
    }
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[builder(default)]
pub struct Arpeggiator {
    mode: ArpeggiatorMode,
    num_octaves: i32,
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

impl Serialize for Polyphony {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Polyphony::Auto => serializer.serialize_str("auto"),
            Polyphony::Mono => serializer.serialize_str("mono"),
            Polyphony::Legato => serializer.serialize_str("legato"),
            Polyphony::Poly => serializer.serialize_str("poly"),
            Polyphony::Integer(x) => serializer.serialize_u32(x),
        }
    }
}

impl<'de> Deserialize<'de> for Polyphony {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "auto" => Ok(Polyphony::Auto),
            "mono" => Ok(Polyphony::Mono),
            "legato" => Ok(Polyphony::Legato),
            "poly" => Ok(Polyphony::Poly),
            other => {
                if let Ok(v) = other.parse::<u32>() {
                    Ok(Polyphony::Integer(v))
                } else {
                    Err(serde::de::Error::custom(other.to_string()))
                }
            }
        }
    }
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Sound {
    osc1: Osc,
    osc2: Osc,
    polyphonic: Polyphony,
    #[serde(default)]
    clipping_amount: u32,
    voice_priority: u32,
    lfo1: Lfo,
    lfo2: Lfo,
    mode: Mode,
    lpf_mode: Option<LpfMode>,
    unison: Unison,
    delay: Delay,
    compressor: Option<Compressor>,
    #[serde(rename = "modFXType")]
    mod_fx_type: ModFxType,
    default_params: DefaultParams,
    #[serde(default)]
    midi_knobs: MidiKnobs,
    #[serde(default)]
    mod_knobs: ModKnobs,
}

impl Default for Sound {
    fn default() -> Sound {
        Sound {
            osc1: Osc::default(),
            osc2: Osc::default(),
            polyphonic: Polyphony::default(),
            clipping_amount: 0,
            voice_priority: 1,
            lfo1: Lfo {
                lfo_type: LfoType::Triangle,
                sync_level: Some(7),
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
            midi_knobs: MidiKnobs::default(),
            mod_knobs: ModKnobs::default(),
        }
    }
}

#[derive(Default, Clone, Builder, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct Synth {
    firmware_version: Option<String>,
    earliest_compatible_firmware: Option<String>,
    sound: Sound,
}

pub fn parse_synth(file: &mut fs::File) -> Synth {
    // Deluge xml files don't have a root node, so add one
    let mut s = "<doc>\n".to_string();
    let _ = file.read_to_string(&mut s).unwrap();
    s.push_str("\n</doc>\n");
    from_str(&s).unwrap()
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
            .transpose(0)
            .cents(0)
            .retrig_phase(Some(-1))
            .build()
            .unwrap();
        let parsed: Osc = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_lfo_type() {
        let s = "<type>sine</type>";
        let expected = LfoType::Sine;
        let parsed: LfoType = from_str(&s).unwrap();
        assert_eq!(parsed, expected);
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
        let s = "<envelope1><attack>0x80000000</attack><decay>0xE6666654</decay><sustain>0x7FFFFFFF</sustain><release>0x80000000</release></envelope1>";
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
        let s = "<patchCable><source>velocity</source><destination>volume</destination><amount>0x3FFFFFE8</amount></patchCable>";
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
        let s = "<patchCables><patchCable><source>velocity</source><destination>volume</destination><amount>0x3FFFFFE8</amount></patchCable></patchCables>";
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
    fn test_build_synth() {
        let synth = SynthBuilder::default()
            .sound(
                SoundBuilder::default()
                    .osc2(
                        OscBuilder::default()
                            .osc_type(OscType::Saw)
                            .build()
                            .unwrap(),
                    )
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        assert_eq!(synth.sound.osc2.osc_type, OscType::Saw);
        println!("{:?}", synth);
    }
}
