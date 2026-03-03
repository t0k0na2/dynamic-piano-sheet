mod bar;
mod note;
mod rectangle;
pub mod soundfont;
pub mod synth;
mod utils;
use bar::Bar;
use note::Note;
use rectangle::Rectangle;
use soundfont::SoundFont;
use std::collections::HashMap;
use synth::SoundSource;
use wasm_bindgen::prelude::*;

use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use web_sys::{
    AudioContext, CanvasRenderingContext2d, ConvolverNode, DelayNode, DynamicsCompressorNode,
    GainNode, OscillatorNode,
};

fn bpm_to_tempo(bpm: f64) -> f64 {
    60000000.0 / bpm
}

fn calc_sec_per_tick(ticks_per_beat: u16, tempo: f64) -> f64 {
    tempo as f64 * 0.000001 / ticks_per_beat as f64
}

const MIDI_CC_NAMES: [&str; 128] = [
    "Bank Select",
    "Modulation Wheel",
    "Breath Controller",
    "Undefined",
    "Foot Controller",
    "Portamento Time",
    "Data Entry (MSB)",
    "Channel Volume",
    "Balance",
    "Undefined",
    "Pan",
    "Expression Controller",
    "Effect Control 1",
    "Effect Control 2",
    "Undefined",
    "Undefined",
    "General Purpose Controller 1",
    "General Purpose Controller 2",
    "General Purpose Controller 3",
    "General Purpose Controller 4",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Bank Select (LSB)",
    "Modulation Wheel (LSB)",
    "Breath Controller (LSB)",
    "Undefined",
    "Foot Controller (LSB)",
    "Portamento Time (LSB)",
    "Data Entry (LSB)",
    "Channel Volume (LSB)",
    "Balance (LSB)",
    "Undefined",
    "Pan (LSB)",
    "Expression Controller (LSB)",
    "Effect Control 1 (LSB)",
    "Effect Control 2 (LSB)",
    "Undefined",
    "Undefined",
    "General Purpose Controller 1 (LSB)",
    "General Purpose Controller 2 (LSB)",
    "General Purpose Controller 3 (LSB)",
    "General Purpose Controller 4 (LSB)",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Sustain Pedal",
    "Portamento On/Off",
    "Sostenuto Pedal",
    "Soft Pedal",
    "Legato Footswitch",
    "Hold 2 Pedal",
    "Sound Variation",
    "Timbre/Harmonic Content",
    "Release Time",
    "Attack Time",
    "Brightness",
    "Sound Controller 6",
    "Sound Controller 7",
    "Sound Controller 8",
    "Sound Controller 9",
    "Sound Controller 10",
    "General Purpose Controller 5",
    "General Purpose Controller 6",
    "General Purpose Controller 7",
    "General Purpose Controller 8",
    "Portamento Control",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Effects 1 Depth (Reverb)",
    "Effects 2 Depth (Tremolo)",
    "Effects 3 Depth (Chorus)",
    "Effects 4 Depth (Celeste)",
    "Effects 5 Depth (Phaser)",
    "Data Increment",
    "Data Decrement",
    "Non-Registered Parameter Number (LSB)",
    "Non-Registered Parameter Number (MSB)",
    "Registered Parameter Number (LSB)",
    "Registered Parameter Number (MSB)",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "Undefined",
    "All Sound Off",
    "Reset All Controllers",
    "Local Control On/Off",
    "All Notes Off",
    "Omni Mode Off",
    "Omni Mode On",
    "Mono Mode On",
    "Poly Mode On",
];

pub mod midi_cc {
    // MSB (0-31)
    pub const BANK_SELECT_MSB: u8 = 0;
    pub const MODULATION: u8 = 1;
    pub const BREATH_CONTROLLER_MSB: u8 = 2;
    pub const FOOT_CONTROLLER_MSB: u8 = 4;
    pub const PORTAMENTO_TIME_MSB: u8 = 5;
    pub const DATA_ENTRY_MSB: u8 = 6;
    pub const CHANNEL_VOLUME: u8 = 7;
    pub const BALANCE_MSB: u8 = 8;
    pub const PAN: u8 = 10;
    pub const EXPRESSION: u8 = 11;
    pub const EFFECT_CONTROL_1_MSB: u8 = 12;
    pub const EFFECT_CONTROL_2_MSB: u8 = 13;
    pub const GENERAL_PURPOSE_1_MSB: u8 = 16;
    pub const GENERAL_PURPOSE_2_MSB: u8 = 17;
    pub const GENERAL_PURPOSE_3_MSB: u8 = 18;
    pub const GENERAL_PURPOSE_4_MSB: u8 = 19;

    // LSB (32-63)
    pub const BANK_SELECT_LSB: u8 = 32;
    pub const MODULATION_LSB: u8 = 33;
    pub const BREATH_CONTROLLER_LSB: u8 = 34;
    pub const FOOT_CONTROLLER_LSB: u8 = 36;
    pub const PORTAMENTO_TIME_LSB: u8 = 37;
    pub const DATA_ENTRY_LSB: u8 = 38;
    pub const CHANNEL_VOLUME_LSB: u8 = 39;
    pub const BALANCE_LSB: u8 = 40;
    pub const PAN_LSB: u8 = 42;
    pub const EXPRESSION_LSB: u8 = 43;
    pub const EFFECT_CONTROL_1_LSB: u8 = 44;
    pub const EFFECT_CONTROL_2_LSB: u8 = 45;
    pub const GENERAL_PURPOSE_1_LSB: u8 = 48;
    pub const GENERAL_PURPOSE_2_LSB: u8 = 49;
    pub const GENERAL_PURPOSE_3_LSB: u8 = 50;
    pub const GENERAL_PURPOSE_4_LSB: u8 = 51;

    // Switch (64-69)
    pub const SUSTAIN_PEDAL: u8 = 64; // Damper Pedal
    pub const PORTAMENTO_SWITCH: u8 = 65;
    pub const SOSTENUTO_PEDAL: u8 = 66;
    pub const SOFT_PEDAL: u8 = 67;
    pub const LEGATO_FOOTSWITCH: u8 = 68;
    pub const HOLD_2_PEDAL: u8 = 69;

    // Sound Controllers (70-79)
    pub const SOUND_VARIATION: u8 = 70;
    pub const TIMBRE_HARMONIC_INTENSITY: u8 = 71;
    pub const RELEASE_TIME: u8 = 72;
    pub const ATTACK_TIME: u8 = 73;
    pub const BRIGHTNESS: u8 = 74;
    pub const SOUND_CONTROLLER_6: u8 = 75;
    pub const SOUND_CONTROLLER_7: u8 = 76;
    pub const SOUND_CONTROLLER_8: u8 = 77;
    pub const SOUND_CONTROLLER_9: u8 = 78;
    pub const SOUND_CONTROLLER_10: u8 = 79;

    // General Purpose 5-8 (80-83)
    pub const GENERAL_PURPOSE_5: u8 = 80;
    pub const GENERAL_PURPOSE_6: u8 = 81;
    pub const GENERAL_PURPOSE_7: u8 = 82;
    pub const GENERAL_PURPOSE_8: u8 = 83;

    // Portamento Control (84)
    pub const PORTAMENTO_CONTROL: u8 = 84;

    // Effects (91-95)
    pub const REVERB: u8 = 91; // Effects 1 Depth
    pub const TREMOLO: u8 = 92; // Effects 2 Depth
    pub const CHORUS: u8 = 93; // Effects 3 Depth
    pub const CELESTE: u8 = 94; // Effects 4 Depth
    pub const PHASER: u8 = 95; // Effects 5 Depth

    // Data Increment/Decrement (96-97)
    pub const DATA_INCREMENT: u8 = 96;
    pub const DATA_DECREMENT: u8 = 97;

    // Parameters (98-101)
    pub const NRPN_LSB: u8 = 98;
    pub const NRPN_MSB: u8 = 99;
    pub const RPN_LSB: u8 = 100;
    pub const RPN_MSB: u8 = 101;

    // Channel Mode Messages (120-127)
    pub const ALL_SOUND_OFF: u8 = 120;
    pub const RESET_ALL_CONTROLLERS: u8 = 121;
    pub const LOCAL_CONTROL: u8 = 122;
    pub const ALL_NOTES_OFF: u8 = 123;
    pub const OMNI_MODE_OFF: u8 = 124;
    pub const OMNI_MODE_ON: u8 = 125;
    pub const MONO_MODE_ON: u8 = 126;
    pub const POLY_MODE_ON: u8 = 127;
}

pub fn parse_midi(data: &[u8]) -> Result<(Vec<Bar>, Vec<Note>, u8), String> {
    let smf = match Smf::parse(data) {
        Ok(smf) => smf,
        Err(e) => {
            log!("Error parsing MIDI file: {:?}", e);
            return Err(format!("Failed to parse MIDI file"));
        }
    };

    if smf.header.format != Format::Parallel {
        return Err(format!("Parallelだけサポート").into());
    }

    let ticks_per_beat = match smf.header.timing {
        Timing::Timecode(_, _) => return Err(format!("タイムコードは未サポート")),
        Timing::Metrical(res) => res.as_int(),
    };

    #[derive(Clone, Copy)]
    struct TrackState {
        interval_ticks: u32,
        currrent_index: usize,
        ended: bool,
        program: u8,
        bank_msb: u8,
        bank_lsb: u8,
        volume: u8,
        pitch_bend: i16,
        pan: u8,
        reverb: u8,
        chorus: u8,
        rpn_lsb: u8,
        rpn_msb: u8,
        data_entry_lsb: u8,
        data_entry_msb: u8,
        modulation: u8,
        expression: u8,
        pitch_bend_sensitivity: f32,
    }

    impl Default for TrackState {
        fn default() -> Self {
            Self {
                interval_ticks: 0,
                currrent_index: 0,
                ended: false,
                program: 0,
                bank_msb: 0,
                bank_lsb: 0,
                volume: 0,
                pitch_bend: 0,
                pan: 64, // Center default
                reverb: 0,
                chorus: 0,
                rpn_lsb: 127,
                rpn_msb: 127,
                data_entry_lsb: 0,
                data_entry_msb: 2,
                modulation: 0,
                expression: 127,
                pitch_bend_sensitivity: 2.0,
            }
        }
    }
    let mut track_states: Vec<TrackState> = vec![TrackState::default(); smf.tracks.len()];

    for (i, track) in smf.tracks.iter().enumerate() {
        if track.is_empty() {
            continue;
        }
        track_states[i].currrent_index = 0;
        track_states[i].interval_ticks = track[track_states[i].currrent_index].delta.as_int();
    }

    let mut ticks_per_bar: usize = (ticks_per_beat * 4) as usize; // デフォルトは4/4拍子にしとく
    let mut sec_per_tick = calc_sec_per_tick(ticks_per_beat, bpm_to_tempo(120.0)); // とりあえず初期テンポ120BPM
    let mut current_time: f64 = 0.0;
    let mut remain_bar_ticks = 0;
    let mut bars: Vec<Bar> = Vec::new();
    let mut notes: Vec<Note> = Vec::new();
    let mut playing_notes: HashMap<(u8, u8), usize> = HashMap::new();

    while track_states.iter().any(|&state| state.ended == false) {
        // 小節情報
        if remain_bar_ticks <= 0 {
            if let Some(bar) = bars.last_mut() {
                bar.set_end_time(current_time);
            }
            bars.push(Bar::new(current_time, -1.0, bars.len() as u32));

            remain_bar_ticks = ticks_per_bar;
        }

        for (i, track) in smf.tracks.iter().enumerate() {
            let track_state = &mut track_states[i];

            if track_state.ended {
                continue;
            }

            // 同じタイミングで複数のイベントが発生することがあるのでループで処理する
            while track_state.currrent_index < track.len() {
                if track_state.interval_ticks > 0 {
                    track_state.interval_ticks -= 1;
                    break;
                }

                match track[track_state.currrent_index].kind {
                    TrackEventKind::Midi { channel, message } => {
                        match message {
                            MidiMessage::NoteOn { key, vel } => {
                                let hash_key = (channel.as_int(), key.as_int());
                                if vel > 0 {
                                    let note_id = notes.len();
                                    notes.push(Note::new(
                                        current_time,
                                        -1.0,
                                        key.as_int(),
                                        vel.as_int(),
                                        track_state.volume,
                                        track_state.pitch_bend,
                                        track_state.pan,
                                        track_state.reverb,
                                        track_state.chorus,
                                        track_state.modulation,
                                        track_state.expression,
                                        track_state.pitch_bend_sensitivity,
                                        i as u8,
                                        track_state.program,
                                        track_state.bank_msb as u16, // SoundFontはmsbのみを使用
                                    ));
                                    if let Some(id) = playing_notes.insert(hash_key, note_id) {
                                        notes[id].set_off_time(current_time);
                                    }
                                } else {
                                    // vel0はNoteOff扱い?
                                    if let Some(id) = playing_notes.remove(&hash_key) {
                                        notes[id].set_off_time(current_time);
                                    }
                                }
                            }
                            MidiMessage::NoteOff { key, .. } => {
                                let hash_key = (channel.as_int(), key.as_int());
                                if let Some(id) = playing_notes.remove(&hash_key) {
                                    notes[id].set_off_time(current_time);
                                }
                            }
                            MidiMessage::ProgramChange { program } => {
                                track_state.program = program.as_int();
                            }
                            MidiMessage::Controller { controller, value } => {
                                match controller.as_int() {
                                    midi_cc::BANK_SELECT_MSB => {
                                        track_state.bank_msb = u8::from(value)
                                    }
                                    midi_cc::BANK_SELECT_LSB => {
                                        track_state.bank_lsb = u8::from(value)
                                    }
                                    midi_cc::DATA_ENTRY_MSB => {
                                        track_state.data_entry_msb = u8::from(value);
                                        if track_state.rpn_msb == 0 && track_state.rpn_lsb == 0 {
                                            track_state.pitch_bend_sensitivity =
                                                track_state.data_entry_msb as f32
                                                    + track_state.data_entry_lsb as f32 / 100.0;
                                        }
                                    }
                                    midi_cc::DATA_ENTRY_LSB => {
                                        track_state.data_entry_lsb = u8::from(value);
                                        if track_state.rpn_msb == 0 && track_state.rpn_lsb == 0 {
                                            track_state.pitch_bend_sensitivity =
                                                track_state.data_entry_msb as f32
                                                    + track_state.data_entry_lsb as f32 / 100.0;
                                        }
                                    }
                                    midi_cc::CHANNEL_VOLUME => {
                                        let vol = u8::from(value);
                                        track_state.volume = vol;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id]
                                                    .add_channel_volume(current_time, vol);
                                            }
                                        }
                                    }
                                    midi_cc::MODULATION => {
                                        let mod_val = u8::from(value);
                                        track_state.modulation = mod_val;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id]
                                                    .add_modulation(current_time, mod_val);
                                            }
                                        }
                                    }
                                    midi_cc::PAN => {
                                        let pan_val = u8::from(value);
                                        track_state.pan = pan_val;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id].add_pan(current_time, pan_val);
                                            }
                                        }
                                    }
                                    midi_cc::EXPRESSION => {
                                        let expr_val = u8::from(value);
                                        track_state.expression = expr_val;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id]
                                                    .add_expression(current_time, expr_val);
                                            }
                                        }
                                    }
                                    midi_cc::REVERB => {
                                        let rv_val = u8::from(value);
                                        track_state.reverb = rv_val;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id].add_reverb(current_time, rv_val);
                                            }
                                        }
                                    }
                                    midi_cc::CHORUS => {
                                        let ch_val = u8::from(value);
                                        track_state.chorus = ch_val;
                                        let ch_id = channel.as_int();
                                        for (&(ch, _), &note_id) in playing_notes.iter() {
                                            if ch == ch_id {
                                                notes[note_id].add_chorus(current_time, ch_val);
                                            }
                                        }
                                    }
                                    midi_cc::NRPN_LSB => {
                                        track_state.rpn_lsb = 127;
                                        track_state.rpn_msb = 127;
                                    }
                                    midi_cc::NRPN_MSB => {
                                        track_state.rpn_lsb = 127;
                                        track_state.rpn_msb = 127;
                                    }
                                    midi_cc::RPN_LSB => track_state.rpn_lsb = u8::from(value),
                                    midi_cc::RPN_MSB => track_state.rpn_msb = u8::from(value),
                                    midi_cc::RESET_ALL_CONTROLLERS => {
                                        track_state.bank_msb = 0;
                                        track_state.bank_lsb = 0;
                                        track_state.volume = 100;
                                        track_state.pitch_bend = 0;
                                        track_state.pan = 64;
                                        track_state.reverb = 0;
                                        track_state.chorus = 0;
                                        track_state.modulation = 0;
                                        track_state.expression = 127;
                                        track_state.rpn_lsb = 127;
                                        track_state.rpn_msb = 127;
                                        track_state.pitch_bend_sensitivity = 2.0;
                                    }
                                    _ => {
                                        let cc_name = MIDI_CC_NAMES
                                            .get(controller.as_int() as usize)
                                            .unwrap_or(&"Unknown");
                                        crate::log!(
                                            "controller: {} ({:?}) {:?}",
                                            cc_name,
                                            controller,
                                            value
                                        );
                                    }
                                }
                            }
                            MidiMessage::PitchBend { bend } => {
                                let bend_val = bend.as_int() as i16;
                                track_state.pitch_bend = bend_val;
                                let ch_id = channel.as_int();
                                for (&(ch, _), &note_id) in playing_notes.iter() {
                                    if ch == ch_id {
                                        notes[note_id].add_pitch_bend(current_time, bend_val);
                                    }
                                }
                            }
                            MidiMessage::Aftertouch { key, vel } => {
                                crate::log!("aftertouch: key{:?} vel{:?}", key, vel);
                            }
                            MidiMessage::ChannelAftertouch { vel } => {
                                crate::log!("channel aftertouch: {:?}", vel);
                            }
                        }
                    }
                    TrackEventKind::Meta(message) => match message {
                        MetaMessage::Tempo(tempo) => {
                            sec_per_tick = calc_sec_per_tick(ticks_per_beat, tempo.as_int() as f64);
                        }
                        MetaMessage::EndOfTrack => {
                            track_state.ended = true;
                        }
                        MetaMessage::TimeSignature(num, denom, _, _) => {
                            if denom != 0 {
                                ticks_per_bar = (ticks_per_beat as f64
                                    / (2u32.pow(denom as u32) as f64 / 4.0) as f64
                                    * num as f64)
                                    as usize;
                                remain_bar_ticks = ticks_per_bar;
                            } else {
                            }
                        }
                        MetaMessage::DeviceName(_name) => {
                            // 再生には不要な情報なので無視してOK
                            //crate::log!("device name: {:?}", name);
                        }
                        MetaMessage::TrackName(_name) => {
                            // 再生には不要な情報なので無視してOK
                            //crate::log!("track name: {:?}", name);
                        }
                        MetaMessage::KeySignature(_sf, _mi) => {
                            // 再生には不要な情報なので無視してOK
                            //crate::log!("key signature: {:?}", message);
                        }
                        MetaMessage::MidiPort(_port) => {
                            // 再生には不要な情報なので無視してOK
                            //crate::log!("midi port: {:?}", message);
                        }
                        MetaMessage::SmpteOffset(_smpte_offset) => {
                            // 再生には不要な情報なので無視してOK
                            //crate::log!("smpte offset: {:?}", message);
                        }
                        _ => {
                            crate::log!("unsupported meta message: {:?}", message);
                        }
                    },
                    _ => {
                        crate::log!(
                            "unsupported track event kind: {:?}",
                            track[track_state.currrent_index].kind
                        );
                        crate::log!(
                            "unsupported track event kind: {:?}",
                            track[track_state.currrent_index].kind
                        );
                    }
                }
                track_state.currrent_index += 1;
                if track_state.currrent_index < track.len() {
                    track_state.interval_ticks = track[track_state.currrent_index].delta.as_int();
                }
            }
        }

        remain_bar_ticks -= 1;
        current_time += sec_per_tick;
    }

    if let Some(bar) = bars.last_mut() {
        bar.set_end_time(current_time + remain_bar_ticks as f64 * sec_per_tick);
    }

    Ok((bars, notes, smf.tracks.len() as u8))
}

fn calc_key_area(rect: &Rectangle, min_key: u8, max_key: u8) -> Vec<Rectangle> {
    // キーボードの１オクターブ分の鍵盤の比率位置テーブルを作成、黒鍵は白鍵にかぶさる上に幅や位置が等幅ではないので定義して使うことにした
    const OCTAVE_SIZE_RATIO_TABLE: [(f64, f64); 12] = [
        (0.0, 0.1428),
        (0.0951, 0.1666),
        (0.1428, 0.2857),
        (0.2618, 0.3333),
        (0.2857, 0.4285),
        (0.4285, 0.5714),
        (0.5237, 0.5952),
        (0.5714, 0.7142),
        (0.6784, 0.7499),
        (0.7142, 0.8571),
        (0.8332, 0.9047),
        (0.8571, 1.0),
    ];

    let min_octave = min_key / 12;
    let min_note = min_key % 12;
    let _max_octave = max_key / 12;
    let max_note = max_key % 12;

    let base_offset_ratio = min_octave as f64 + OCTAVE_SIZE_RATIO_TABLE[min_note as usize].0;
    let octave_width = {
        let min_aligned = min_key + 12 - (min_key % 12);
        let max_aligned = max_key - (max_key % 12);
        let total_octave_ratio = (max_aligned - min_aligned) as f64 / 12.0
            + (1.0 - OCTAVE_SIZE_RATIO_TABLE[min_note as usize].0)
            + OCTAVE_SIZE_RATIO_TABLE[max_note as usize].1;
        rect.width() / total_octave_ratio
    };

    let mut ret: Vec<Rectangle> = Vec::new();
    for key in min_key..=max_key {
        let note_index = (key % 12) as usize;
        let octave = (key / 12) as f64;
        let left = rect.left()
            + (octave + OCTAVE_SIZE_RATIO_TABLE[note_index].0 - base_offset_ratio) * octave_width;
        let right = rect.left()
            + (octave + OCTAVE_SIZE_RATIO_TABLE[note_index].1 - base_offset_ratio) * octave_width;
        ret.push(Rectangle::new(
            left,
            rect.top(),
            right - left,
            rect.height(),
        ));
    }
    ret
}

pub struct ChorusNode {
    input: GainNode,
    output: GainNode,
    _delay: DelayNode,
    _lfo: OscillatorNode,
    _lfo_depth: GainNode,
    _wet_gain: GainNode,
}

impl ChorusNode {
    pub fn new(
        audio_ctx: &AudioContext,
        rate: Option<f32>,
        depth: Option<f32>,
        delay_time: Option<f64>,
        wet: Option<f32>,
    ) -> Result<Self, JsValue> {
        let rate = rate.unwrap_or(1.5);
        let depth = depth.unwrap_or(0.002);
        let delay_time = delay_time.unwrap_or(0.02);
        let wet = wet.unwrap_or(1.0);

        let input = audio_ctx.create_gain()?;
        let output = audio_ctx.create_gain()?;

        let wet_gain = audio_ctx.create_gain()?;

        let delay = audio_ctx.create_delay()?;
        let lfo = audio_ctx.create_oscillator()?;
        let lfo_depth = audio_ctx.create_gain()?;

        delay.delay_time().set_value(delay_time as f32);
        lfo.set_type(web_sys::OscillatorType::Sine);
        lfo.frequency().set_value(rate);
        lfo_depth.gain().set_value(depth);
        wet_gain.gain().set_value(wet);

        input.connect_with_audio_node(&delay)?;
        delay.connect_with_audio_node(&wet_gain)?;
        wet_gain.connect_with_audio_node(&output)?;

        lfo.connect_with_audio_node(&lfo_depth)?;
        lfo_depth.connect_with_audio_param(&delay.delay_time())?;
        lfo.start()?;

        Ok(Self {
            input,
            output,
            _delay: delay,
            _lfo: lfo,
            _lfo_depth: lfo_depth,
            _wet_gain: wet_gain,
        })
    }

    pub fn input(&self) -> &GainNode {
        &self.input
    }

    pub fn connect_with_audio_node(
        &self,
        target: &web_sys::AudioNode,
    ) -> Result<web_sys::AudioNode, JsValue> {
        self.output.connect_with_audio_node(target)
    }
}

#[wasm_bindgen]
pub struct MidiPlayer {
    audio_context: AudioContext,
    reverb: ConvolverNode,
    chorus: ChorusNode,
    master_volume: GainNode,
    sound_sources: Vec<SoundSource>,
    bars: Vec<Bar>,
    notes: Vec<Note>,
    current_time: f64,
    playing: bool,
    display_range_sec: f64,
    num_tracks: u8,
    loop_start_bar: usize,
    loop_end_bar: usize,
    soundfont: Option<SoundFont>,
}

pub fn create_rich_synthesized_reverb(
    audio_ctx: &AudioContext,
    duration: Option<f64>,
    decay: Option<f64>,
    pre_delay_ms: Option<f64>,
) -> Result<ConvolverNode, JsValue> {
    let duration = duration.unwrap_or(2.0);
    let decay = decay.unwrap_or(2.0);
    let pre_delay_ms = pre_delay_ms.unwrap_or(30.0);

    let sample_rate = audio_ctx.sample_rate();
    let length = (sample_rate as f64 * duration) as u32;
    // create_bufferの引数は (numOfChannels, length, sampleRate) です
    let impulse = audio_ctx.create_buffer(2, length, sample_rate)?;

    let pre_delay_samples = ((pre_delay_ms / 1000.0) * sample_rate as f64).floor() as u32;

    for i in 0..2 {
        let mut channel_data = vec![0.0f32; length as usize];
        let mut last_out = 0.0;

        for j in 0..length {
            // 1. プリディレイ：指定サンプル数までは無音（0）にする
            if j < pre_delay_samples {
                channel_data[j as usize] = 0.0;
                continue;
            }

            // 2. 指数関数的な減衰カーブ
            let multiplier = (1.0
                - (j - pre_delay_samples) as f64 / (length - pre_delay_samples) as f64)
                .powf(decay);

            // 3. ホワイトノイズの生成
            let white_noise = (js_sys::Math::random() * 2.0 - 1.0) * multiplier;

            // 4. シンプルな1次ローパスフィルター（高域を削って温かみを出す）
            // 係数(0.2)を小さくするほど、よりこもった音（Dampが強い音）になります
            last_out = last_out + 0.2 * (white_noise - last_out);

            channel_data[j as usize] = last_out as f32;
        }

        impulse.copy_to_channel(&mut channel_data, i as i32)?;
    }

    let convolver = audio_ctx.create_convolver()?;
    convolver.set_buffer(Some(&impulse));
    Ok(convolver)
}

#[wasm_bindgen]
impl MidiPlayer {
    pub fn new() -> Result<MidiPlayer, JsValue> {
        utils::set_panic_hook();
        let audio_context = AudioContext::new()?;

        let master_volume = audio_context.create_gain()?;
        master_volume.connect_with_audio_node(&audio_context.destination())?;

        let reverb = create_rich_synthesized_reverb(&audio_context, None, None, None)?;
        reverb.connect_with_audio_node(&master_volume)?;

        let chorus = ChorusNode::new(&audio_context, None, None, None, None)?;
        chorus.connect_with_audio_node(&master_volume)?;

        Ok(MidiPlayer {
            audio_context: audio_context,
            reverb: reverb,
            chorus: chorus,
            master_volume: master_volume,
            bars: Vec::new(),
            notes: Vec::new(),
            current_time: 0.0,
            sound_sources: Vec::new(),
            playing: false,
            display_range_sec: 3.0,
            num_tracks: 0,
            loop_start_bar: 0,
            loop_end_bar: 0,
            soundfont: None,
        })
    }

    pub fn load_midi(&mut self, bin: &[u8]) -> Result<(), JsValue> {
        let parse_result = match parse_midi(bin) {
            Ok(parse_result) => parse_result,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Error parsing MIDI file: {}",
                    e
                )));
            }
        };

        self.playing = false;
        self.current_time = 0.0;
        self.sound_sources.clear();
        self.bars = parse_result.0;
        self.notes = parse_result.1;
        self.num_tracks = parse_result.2;

        Ok(())
    }

    pub fn load_soundfont(&mut self, bin: &[u8]) -> Result<(), JsValue> {
        let sf = match SoundFont::parse(bin) {
            Ok(sf) => sf,
            Err(e) => {
                return Err(JsValue::from_str(&format!(
                    "Error parsing SoundFont: {}",
                    e
                )));
            }
        };

        if sf.sample_data.is_empty() {
            return Err(JsValue::from_str("No sample data in SoundFont"));
        }

        self.soundfont = Some(sf);

        Ok(())
    }

    pub fn current_playback_time(&self) -> f64 {
        self.current_time
    }

    pub fn song_length(&self) -> f64 {
        match self.bars.last() {
            Some(bar) => bar.end_time(),
            None => 0.0,
        }
    }

    pub fn play(&mut self) {
        if self.ready() == false {
            return;
        }
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.sound_sources.clear();
    }

    pub fn ready(&self) -> bool {
        self.notes.len() > 0 && self.bars.len() > 0
    }

    pub fn set_loop_bars(&mut self, start_bar: usize, end_bar: usize) {
        self.loop_start_bar = start_bar;
        self.loop_end_bar = end_bar;
    }

    pub fn num_bars(&self) -> usize {
        self.bars.len()
    }

    pub fn volume(&self) -> f32 {
        self.master_volume.gain().value()
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.master_volume.gain().set_value(volume);
    }

    pub fn current_bar(&self) -> usize {
        if self.bars.len() == 0 {
            return 0;
        }

        if self.current_time < self.bars[0].begin_time() {
            return 0;
        }

        for bar in self.bars.iter() {
            if bar.begin_time() <= self.current_time && self.current_time < bar.end_time() {
                return bar.number() as usize;
            }
        }
        self.bars.len()
    }

    pub fn set_display_range(&mut self, range_sec: f64) {
        self.display_range_sec = range_sec;
    }

    pub fn seek_bar(&mut self, bar: usize, clear_sounds: bool) {
        let bar = &self.bars[bar.clamp(0, self.bars.len() - 1)];
        self.seek_time(bar.begin_time(), clear_sounds);
    }

    pub fn seek_time(&mut self, time: f64, clear_sounds: bool) {
        if clear_sounds {
            self.sound_sources.clear();
        }
        self.current_time = time.clamp(0.0, self.song_length());
    }

    pub fn skip(&mut self, delta: f64, clear_sounds: bool) {
        self.seek_time(self.current_time + delta, clear_sounds);
    }

    pub fn tick(&mut self, delta_time: f64) -> Result<(), JsValue> {
        if self.playing == false {
            return Ok(());
        }

        let delta_sec = delta_time / 1000.0; // ms -> s

        for sound_source in self.sound_sources.iter_mut() {
            sound_source.tick(delta_sec);
        }
        self.sound_sources.retain(|source| !source.finished());

        let play_reserve_buf_sec = delta_sec;
        for note in self.notes.iter() {
            if self.current_time <= note.on_time()
                && note.on_time() < self.current_time + play_reserve_buf_sec
            {
                let start_time =
                    self.audio_context.current_time() + (note.on_time() - self.current_time);
                let end_time = start_time + (note.off_time() - note.on_time());
                self.sound_sources.push(SoundSource::new(
                    &self.audio_context,
                    &self.master_volume,
                    &self.reverb,
                    self.chorus.input(),
                    note.key(),
                    note.velocity(),
                    note.channel_volumes(),
                    note.pitch_bends(),
                    note.pans(),
                    note.reverbs(),
                    note.choruses(),
                    note.modulations(),
                    note.expressions(),
                    note.pitch_bend_sensitivity(),
                    note.track() + 1,
                    note.program(),
                    note.bank(),
                    start_time,
                    end_time,
                    self.soundfont.as_ref(),
                )?);
            }
        }

        self.current_time += delta_sec;

        if self.loop_end_bar > self.loop_start_bar {
            if self.current_time >= self.bars[self.loop_end_bar].end_time() {
                let loop_start_time = self.bars[self.loop_start_bar].begin_time()
                    - (self.current_time - self.bars[self.loop_end_bar].end_time());
                self.seek_time(loop_start_time, true);
            }
        }

        if self.current_time >= self.song_length() {
            self.playing = false;
            self.current_time = self.song_length() - 0.0001;
        }

        Ok(())
    }

    pub fn render(
        &self,
        context: &CanvasRenderingContext2d,
        left: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Result<(), JsValue> {
        let keybord_height = height * 0.1;
        let min_key: u8 = 21;
        let max_key: u8 = 108;
        let rect = Rectangle::new(left, top, width, height);
        let key_areas = calc_key_area(&rect, min_key, max_key);

        // 背景
        context.set_fill_style_str("black");
        context.fill_rect(rect.left(), rect.top(), rect.width(), rect.height());

        // オクターブ分割線
        context.set_stroke_style_str("gray");
        for key in min_key..=max_key {
            if key % 12 == 0 {
                let area = &key_areas[(key - min_key) as usize];
                context.begin_path();
                context.move_to(area.left(), area.top());
                context.line_to(area.left(), area.bottom());
                context.stroke();
            }
        }

        let display_start_sec = self.current_time;
        let display_end_sec = display_start_sec + self.display_range_sec;
        let pixel_per_sec = rect.height() / self.display_range_sec;
        let current_time_pos = rect.height() - keybord_height;

        // 小節線描画
        context.set_stroke_style_str("gray");
        context.set_fill_style_str("gray");
        context.set_text_align("right");
        context.set_text_baseline("bottom");
        context.set_font("32px sans-serif");
        for bar in self.bars.iter() {
            if bar.begin_time() > display_end_sec || bar.end_time() < display_start_sec {
                continue;
            }
            let bar_pos = current_time_pos - (bar.begin_time() - self.current_time) * pixel_per_sec;
            context.begin_path();
            context.move_to(rect.left(), bar_pos);
            context.line_to(rect.right(), bar_pos);
            context.stroke();
            context.fill_text(
                &(bar.number() + 1).to_string(),
                rect.right() - 2.0,
                bar_pos - 2.0,
            )?;
            if bar.number() == self.bars.len() as u32 - 1 {
                // 最後の小節線も描画
                let end_bar_pos =
                    current_time_pos - (bar.end_time() - self.current_time) * pixel_per_sec;
                context.begin_path();
                context.move_to(rect.left(), end_bar_pos);
                context.line_to(rect.right(), end_bar_pos);
                context.stroke();
                context.fill_text("おわり", rect.right() - 2.0, end_bar_pos - 2.0)?;
            }
        }

        const TRACK_FILL_COLORS: [&str; 4] = ["#4682B4", "#E66101", "#009E73", "#7B4173"];
        const TRACK_STROKE_COLORS: [&str; 4] = ["#266294", "#C64101", "#007E53", "#5B2153"];

        // ノート描画
        let diplay_notes: Vec<&Note> = self
            .notes
            .iter()
            .filter(|note| {
                note.on_time() <= display_end_sec
                    && display_start_sec <= note.off_time()
                    && min_key <= note.key()
                    && note.key() <= max_key
            })
            .collect();

        for track_no in 0..self.num_tracks {
            let color_index = (track_no as usize % TRACK_FILL_COLORS.len()) as usize;
            context.set_stroke_style_str(TRACK_STROKE_COLORS[color_index]);
            context.set_fill_style_str(TRACK_FILL_COLORS[color_index]);

            for note in diplay_notes.iter() {
                if note.track() != track_no {
                    continue;
                }

                let area = &key_areas[(note.key() - min_key) as usize];
                let note_top =
                    current_time_pos - (note.off_time() - self.current_time) * pixel_per_sec;
                let note_height = current_time_pos
                    - (note.on_time() - self.current_time) * pixel_per_sec
                    - note_top;
                let note_left = area.left();
                let note_width = area.width();

                context.begin_path();
                context.round_rect_with_f64(note_left, note_top, note_width, note_height, 4.0)?;
                context.fill();
                context.stroke();
            }
        }

        let playing_diplay_notes: Vec<&Note> = self
            .notes
            .iter()
            .filter(|note| {
                note.on_time() <= self.current_time
                    && self.current_time <= note.off_time()
                    && min_key <= note.key()
                    && note.key() <= max_key
            })
            .collect();

        // 白鍵
        let white_note_height = keybord_height;
        context.set_stroke_style_str("gray");
        context.set_fill_style_str("white");
        for key in min_key..=max_key {
            match key % 12 {
                0 | 2 | 4 | 5 | 7 | 9 | 11 => {
                    let area = &key_areas[(key - min_key) as usize];
                    let top = area.bottom() - white_note_height;
                    context.fill_rect(area.left(), top, area.width(), white_note_height);
                    context.begin_path();
                    context.move_to(area.left(), top);
                    context.line_to(area.left(), area.bottom());
                    context.stroke();
                }
                _ => (),
            }
        }

        // 再生している白鍵
        for track_no in 0..self.num_tracks {
            let color_index = (track_no as usize % TRACK_FILL_COLORS.len()) as usize;
            context.set_stroke_style_str(TRACK_STROKE_COLORS[color_index]);
            context.set_fill_style_str(TRACK_FILL_COLORS[color_index]);
            for note in playing_diplay_notes.iter() {
                if note.track() != track_no {
                    continue;
                }
                match note.key() % 12 {
                    0 | 2 | 4 | 5 | 7 | 9 | 11 => {
                        let area = &key_areas[(note.key() - min_key) as usize];
                        let top = area.bottom() - white_note_height;
                        context.fill_rect(area.left(), top, area.width(), white_note_height);
                        context.begin_path();
                        context.move_to(area.left(), top);
                        context.line_to(area.left(), area.bottom());
                        context.stroke();
                    }
                    _ => (),
                }
            }
        }

        // 黒鍵
        let black_note_height = keybord_height * 0.6;
        context.set_fill_style_str("black");
        for key in min_key..=max_key {
            match key % 12 {
                1 | 3 | 6 | 8 | 10 => {
                    let area = &key_areas[(key - min_key) as usize];
                    context.fill_rect(
                        area.left(),
                        area.bottom() - white_note_height,
                        area.width(),
                        black_note_height,
                    );
                }
                _ => (),
            }
        }

        // 再生している黒鍵
        for track_no in 0..self.num_tracks {
            let color_index = (track_no as usize % TRACK_FILL_COLORS.len()) as usize;
            context.set_stroke_style_str(TRACK_STROKE_COLORS[color_index]);
            context.set_fill_style_str(TRACK_FILL_COLORS[color_index]);
            for note in playing_diplay_notes.iter() {
                if note.track() != track_no {
                    continue;
                }
                match note.key() % 12 {
                    1 | 3 | 6 | 8 | 10 => {
                        let area = &key_areas[(note.key() - min_key) as usize];
                        context.fill_rect(
                            area.left(),
                            area.bottom() - white_note_height,
                            area.width(),
                            black_note_height,
                        );
                    }
                    _ => (),
                }
            }
        }

        Ok(())
    }
}

mod test {
    #[test]
    fn test_parse_midi() {
        let data = include_bytes!("../tests/assets/test.mid");
        let result = super::parse_midi(data);
        assert!(result.is_ok());

        let midi = result.unwrap();

        /*assert!(midi.1.len() > 0);
        for note in &midi.1 {
            println!("{:?}", note);
        }

        assert!(midi.0.len() > 0);
        for bar in &midi.0 {
            println!("{:?}", bar);
        }*/
    }
}
