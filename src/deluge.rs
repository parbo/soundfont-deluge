use derive_builder::Builder;
use std::fs;
use std::io::{Read, Write};
use yaserde;
use yaserde::de::from_str;
use yaserde::ser::to_string_with_config;

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Value(u32);

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
    file_name: String,
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
}

impl Default for Osc {
    fn default() -> Osc {
        Osc {
            osc_type: OscType::Square,
            transpose: Some(0),
            cents: Some(0),
            retrig_phase: Some(-1),
            loop_mode: None,
            reversed: None,
            time_stretch_enable: None,
            time_stretch_amount: None,
            sample_ranges: None,
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
    midi_knobs: MidiKnobs,
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
            midi_knobs: MidiKnobs::default(),
            mod_knobs: ModKnobs::default(),
        }
    }
}

impl Sound {
    pub fn to_xml(&self) -> String {
        let yaserde_cfg = yaserde::ser::Config {
            perform_indent: true,
            ..Default::default()
        };
        // serialize
        to_string_with_config(self, &yaserde_cfg)
            .unwrap()
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
}
