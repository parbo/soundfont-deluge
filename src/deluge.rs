use serde::{Deserialize, Serialize};
use quick_xml::de::{from_str, DeError};
use std::fs;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum OscType {
    AnalogSaw,
    AnalogSquare,
    Saw,
    Square,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Osc {
    #[serde(rename = "type")]
    osc_type: OscType,
    transpose: i32,
    cents: i32,
    retrig_phase: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LfoType {
    Sine,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Lfo {
    #[serde(rename = "type")]
    lfo_type: LfoType,
    sync_level: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    RingMod,
    Subtractive,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Unison {
    num: i32,
    detune: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Delay {
    ping_pong: i32,
    analog: i32,
    sync_level: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LpfMode {
    #[serde(rename = "24dB")]
    Mode24dB,
    #[serde(rename = "24dBDrive")]
    Mode24dBDrive,
    #[serde(rename = "12dB")]
    Mode12dB,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ModFxType {
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Envelope {
    attack: i32,
    decay: i32,
    sustain: i32,
    release: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Envelope1,
    Envelope2,
    Note,
    Velocity,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Destination {
    DelayFeedback,
    DelayRate,
    Env1Attack,
    Env1Release,
    Lfo1Rate,
    LpfFrequency,
    LpfResonance,
    OscBPhaseWidth,
    Pan,
    Pitch,
    Portamento,
    ReverbAmount,
    StutterRate,
    Volume,
    VolumePostFx,
    VolumePostReverbSend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PatchCable {
    source: Source,
    destination: Destination,
    amount: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Equalizer {
    bass: i32,
    treble: i32,
    bass_frequency: i32,
    treble_ferquency: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DefaultParams {
    arpeggiator_gate: i32,
    portamento: i32,
    compressor_shape: i32,
    osc_a_volume: i32,
    osc_a_pulse_width: i32,
    osc_b_volume: i32,
    osc_b_pulse_width: i32,
    noise_voluem: i32,
    volume: i32,
    pan: i32,
    lpf_frequency: i32,
    lpf_resonance: i32,
    hpf_frequency: i32,
    hpf_resonance: i32,
    envelope_1: Envelope,
    envelope_2: Envelope,
    lfo_1_rate: i32,
    lof_2_rate: i32,
    modulator_1_amount: i32,
    modulator_2_amount: i32,
    carrier_1_feedback: i32,
    carrier_2_feedback: i32,
    mod_fx_rate: i32,
    mod_fx_depth: i32,
    delay_rate: i32,
    delay_feedback: i32,
    reverb_amount: i32,
    arpeggiator_rate: i32,
    patch_cables: Vec<PatchCable>,
    stutter_rate: i32,
    sample_rate_reduction: i32,
    bit_crush: i32,
    equalizer: Equalizer,
    mod_fx_offset: i32,
    mod_fx_feedback: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MidiKnob {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModKnob {
    controls_param: Destination,
    patch_amount_from_source: Option<Source>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    osc1: Osc,
    osc2: Osc,
    polyphonic: i32,
    clipping_amount: i32,
    voice_priority: i32,
    lfo1: Lfo,
    lfo2: Lfo,
    mode: Mode,
    unison: Unison,
    delay: Delay,
    mod_fx_type: ModFxType,
    default_params: DefaultParams,
    midi_knobs: Vec<MidiKnob>,
    mod_knobs: Vec<ModKnob>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Synth {
    firmware_version: Option<String>,
    earliest_compatible_firmware: Option<String>,
    sound: Sound,
}

pub fn parse_synth(file: &mut fs::File) -> Synth {
    let mut s = String::new();
    let _ = file.read_to_string(&mut s).unwrap();
    from_str(&s).unwrap()
}

pub fn parse_sound(file: &mut fs::File) -> Sound {
    let mut s = String::new();
    let _ = file.read_to_string(&mut s).unwrap();
    from_str(&s).unwrap()
}


