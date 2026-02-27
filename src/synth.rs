use crate::soundfont::{GeneratorOperator, SoundFont};
use wasm_bindgen::prelude::*;
use web_sys::{AudioNode, BaseAudioContext};

#[derive(Clone, Copy, Debug)]
pub struct Adsr {
    pub delay: f64,
    pub attack: f64,
    pub hold: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64,
}

pub struct SoundSource {
    nodes: Vec<AudioNode>,
    now_time: f64,
    end_time: f64,
}

impl SoundSource {
    pub fn new(
        context: &BaseAudioContext,
        destination_target: &AudioNode,
        key: u8,
        velocity: u8,
        program: u8,
        bank: u16,
        start_time: f64,
        end_time: f64,
        soundfont: Option<&SoundFont>,
    ) -> Result<SoundSource, JsValue> {
        if let Some(sf) = soundfont {
            Self::new_soundfont(
                context,
                destination_target,
                key,
                velocity,
                program,
                bank,
                start_time,
                end_time,
                sf,
            )
        } else {
            // Fallback to silence if SoundFont data is missing
            Ok(SoundSource {
                nodes: vec![],
                now_time: start_time,
                end_time: start_time,
            })
        }
    }

    fn new_soundfont(
        context: &BaseAudioContext,
        destination_target: &AudioNode,
        key: u8,
        velocity: u8,
        program: u8,
        bank: u16,
        start_time: f64,
        end_time: f64,
        soundfont: &SoundFont,
    ) -> Result<SoundSource, JsValue> {
        let default_adsr = Adsr {
            delay: 0.0,
            attack: 0.01,
            hold: 0.0,
            decay: 0.2,
            sustain: 0.8,
            release: 0.5,
        };

        // 使用するサンプルのインデックスを探す
        let (sample_idx, overriding_root_key, adsr) = Self::find_sample_index(
            soundfont, bank, program, key, velocity,
        )
        .unwrap_or((0, None, default_adsr));

        let _freq = Self::midi_key_to_freq(key);
        let vel_ratio = Self::velocity_to_ratio(velocity);

        // sample_idxが範囲外の場合のフォールバック
        let shdr = if sample_idx < soundfont.sample_headers.len() {
            Some(&soundfont.sample_headers[sample_idx])
        } else if !soundfont.sample_headers.is_empty() {
            Some(&soundfont.sample_headers[0])
        } else {
            None
        };

        let sample_rate = if let Some(h) = shdr {
            h.sample_rate as f32
        } else {
            44100.0
        };

        let mut original_pitch = if let Some(h) = shdr {
            h.original_pitch as f32
        } else {
            60.0
        };

        // overridingRootKey が指定されている場合は優先
        if let Some(root_key) = overriding_root_key {
            // SF2の仕様では0~127が有効とされている
            if root_key <= 127 {
                original_pitch = root_key as f32;
            }
        }

        let playback_rate = 2.0_f32.powf((key as f32 - original_pitch) / 12.0);

        // 動的にAudioBufferを生成する
        let audio_buffer = if let Some(h) = shdr {
            let start_idx = h.start as usize;
            let end_idx = h.end as usize;

            // 安全のためデータサイズの範囲内にする
            let safe_start = start_idx.min(soundfont.sample_data.len());
            let safe_end = end_idx.min(soundfont.sample_data.len()).max(safe_start);
            let safe_len = safe_end - safe_start;

            if safe_len > 0 {
                let buffer = context.create_buffer(1, safe_len as u32, sample_rate)?;
                let mut f32_data = vec![0.0f32; safe_len];
                let src_data = &soundfont.sample_data[safe_start..safe_end];
                for (i, &sample) in src_data.iter().enumerate() {
                    f32_data[i] = sample as f32 / 32768.0;
                }
                buffer.copy_to_channel(&mut f32_data, 0)?;
                Some(buffer)
            } else {
                None
            }
        } else {
            None
        };

        // 1. AudioBufferSourceNode
        let source_node = context.create_buffer_source()?;
        if let Some(buf) = &audio_buffer {
            source_node.set_buffer(Some(buf));
        }
        source_node.playback_rate().set_value(playback_rate);

        // sf2のサンプルヘッダーからループポイント等の情報を取得して設定
        if let Some(h) = shdr {
            source_node.set_loop(true);
            // 動的生成の場合、取り出したAudioBufferサイズに合わせた相対位置に直す
            let rel_loop_start = h.start_loop.saturating_sub(h.start) as f64 / sample_rate as f64;
            let rel_loop_end = h.end_loop.saturating_sub(h.start) as f64 / sample_rate as f64;
            source_node.set_loop_start(rel_loop_start);
            source_node.set_loop_end(rel_loop_end);
        }

        // 2. VCA (Volume Envelope) ADSRを構築
        let vca = context.create_gain()?;
        let vca_gain = vca.gain();

        let mut current_time = start_time;
        vca_gain.set_value_at_time(0.0001, current_time)?;

        if adsr.delay > 0.0 {
            current_time += adsr.delay;
            vca_gain.set_value_at_time(0.0001, current_time)?;
        }

        current_time += adsr.attack;
        vca_gain.exponential_ramp_to_value_at_time(vel_ratio as f32, current_time)?;

        if adsr.hold > 0.0 {
            current_time += adsr.hold;
            vca_gain.set_value_at_time(vel_ratio as f32, current_time)?;
        }

        current_time += adsr.decay;
        let vca_sus = (adsr.sustain * vel_ratio).max(0.0001);
        vca_gain.exponential_ramp_to_value_at_time(vca_sus as f32, current_time)?;
        if end_time > current_time {
            vca_gain.exponential_ramp_to_value_at_time(vca_sus as f32, end_time)?;
        }
        vca_gain.exponential_ramp_to_value_at_time(0.0001, end_time + adsr.release)?;

        // Connection
        source_node.connect_with_audio_node(&vca)?;
        vca.connect_with_audio_node(destination_target)?;

        // Play
        // AudioBufferを切り出しているのでオフセットを0にする
        if let Some(_) = shdr {
            source_node.start_with_when(start_time)?;
        } else {
            source_node.start_with_when(start_time)?;
        }

        #[allow(deprecated)]
        source_node.stop_with_when(end_time + adsr.release + 0.1)?;

        let cleanup_time = end_time + adsr.release + 0.2;

        Ok(SoundSource {
            nodes: vec![source_node.into(), vca.into()],
            now_time: start_time,
            end_time: cleanup_time,
        })
    }

    // keyとvelocity、program(preset)、bankから一致するsample情報を取得する
    fn find_sample_index(
        soundfont: &SoundFont,
        bank: u16,
        program: u8,
        key: u8,
        velocity: u8,
    ) -> Option<(usize, Option<u8>, Adsr)> {
        // 1. 該当のプリセットを検索
        let preset_idx = soundfont
            .preset_headers
            .iter()
            .position(|p| p.preset == program as u16 && p.bank == bank)
            // 該当がなければ bank 変えずに program のみ、あるいは bank 0 にフォールバックなどを検討（ここでは柔軟にヒットさせる）
            .or_else(|| {
                soundfont
                    .preset_headers
                    .iter()
                    .position(|p| p.preset == program as u16)
            })
            .or_else(|| {
                soundfont
                    .preset_headers
                    .iter()
                    .position(|p| p.preset == 0 && p.bank == 0)
            })?;

        let pbag_start = soundfont.preset_headers[preset_idx].preset_bag_ndx as usize;
        let pbag_end = soundfont
            .preset_headers
            .get(preset_idx + 1)
            .map(|p| p.preset_bag_ndx as usize)
            .unwrap_or(soundfont.preset_bags.len());

        let mut matched_instrument_id = None;
        let mut preset_gens = [None; 60];
        let mut p_global_gens = [None; 60];

        // 2. プリセットから一致するゾーン（インストゥルメント）を検索
        for b in pbag_start..pbag_end {
            let gen_start = soundfont.preset_bags[b].gen_ndx as usize;
            let gen_end = soundfont
                .preset_bags
                .get(b + 1)
                .map(|bg| bg.gen_ndx as usize)
                .unwrap_or(soundfont.preset_generators.len());

            let mut key_in_range = true;
            let mut vel_in_range = true;
            let mut inst_id = None;
            let mut local_gens = [None; 60];

            for g in gen_start..gen_end {
                if let Some(generator) = soundfont.preset_generators.get(g) {
                    if (generator.gen_oper as usize) < 60 {
                        local_gens[generator.gen_oper as usize] = Some(generator.gen_amount);
                    }
                    if let Ok(op) = std::convert::TryFrom::try_from(generator.gen_oper) {
                        match op {
                            GeneratorOperator::KeyRange => {
                                // keyRange
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if key < lo || key > hi {
                                    key_in_range = false;
                                }
                            }
                            GeneratorOperator::VelRange => {
                                // velRange
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if velocity < lo || velocity > hi {
                                    vel_in_range = false;
                                }
                            }
                            GeneratorOperator::Instrument => {
                                // instrument
                                inst_id = Some(generator.gen_amount as usize);
                            }
                            _ => {}
                        }
                    }
                }
            }

            if inst_id.is_none() && b == pbag_start {
                p_global_gens = local_gens;
                continue;
            }

            // 条件に一致かつインストゥルメントIDが見つかった場合
            if key_in_range && vel_in_range {
                if let Some(id) = inst_id {
                    matched_instrument_id = Some(id);
                    preset_gens = local_gens;
                    break;
                }
            }
        }

        let inst_id = matched_instrument_id?;

        // 3. インストゥルメントから一致するサンプルを検索
        let ibag_start = soundfont.instruments.get(inst_id)?.inst_bag_ndx as usize;
        let ibag_end = soundfont
            .instruments
            .get(inst_id + 1)
            .map(|i| i.inst_bag_ndx as usize)
            .unwrap_or(soundfont.instrument_bags.len());

        let mut i_global_gens = [None; 60];

        for b in ibag_start..ibag_end {
            let gen_start = soundfont.instrument_bags.get(b)?.inst_gen_ndx as usize;
            let gen_end = soundfont
                .instrument_bags
                .get(b + 1)
                .map(|bg| bg.inst_gen_ndx as usize)
                .unwrap_or(soundfont.instrument_generators.len());

            let mut key_in_range = true;
            let mut vel_in_range = true;
            let mut sample_id = None;
            let mut overriding_root_key = None;
            let mut local_gens = [None; 60];

            for g in gen_start..gen_end {
                if let Some(generator) = soundfont.instrument_generators.get(g) {
                    if (generator.gen_oper as usize) < 60 {
                        local_gens[generator.gen_oper as usize] = Some(generator.gen_amount);
                    }
                    if let Ok(op) = std::convert::TryFrom::try_from(generator.gen_oper) {
                        match op {
                            GeneratorOperator::KeyRange => {
                                // keyRange
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if key < lo || key > hi {
                                    key_in_range = false;
                                }
                            }
                            GeneratorOperator::VelRange => {
                                // velRange
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if velocity < lo || velocity > hi {
                                    vel_in_range = false;
                                }
                            }
                            GeneratorOperator::SampleID => {
                                // sampleID
                                sample_id = Some(generator.gen_amount as usize);
                            }
                            GeneratorOperator::OverridingRootKey => {
                                // overridingRootKey
                                overriding_root_key = Some(generator.gen_amount as u8);
                            }
                            _ => {}
                        }
                    }
                }
            }

            if sample_id.is_none() && b == ibag_start {
                i_global_gens = local_gens;
                continue;
            }

            if key_in_range && vel_in_range {
                if let Some(id) = sample_id {
                    let get_gen = |oper: GeneratorOperator, default: i16| -> i16 {
                        let op_idx = oper as usize;
                        // Priority: Instrument Local > Instrument Global
                        let inst_val = local_gens[op_idx]
                            .or(i_global_gens[op_idx])
                            .unwrap_or(default);
                        // Add Preset Local > Preset Global (default offset 0)
                        let preset_val = preset_gens[op_idx].or(p_global_gens[op_idx]).unwrap_or(0);
                        inst_val.saturating_add(preset_val)
                    };

                    let delay = get_gen(GeneratorOperator::DelayVolEnv, -12000);
                    let attack = get_gen(GeneratorOperator::AttackVolEnv, -12000);
                    let hold = get_gen(GeneratorOperator::HoldVolEnv, -12000);
                    let decay = get_gen(GeneratorOperator::DecayVolEnv, -12000);
                    let sustain = get_gen(GeneratorOperator::SustainVolEnv, 0);
                    let release = get_gen(GeneratorOperator::ReleaseVolEnv, -12000);

                    // 1200 cents = 1 octave = factor of 2
                    let delay_sec = if delay <= -12000 {
                        0.0
                    } else {
                        2.0_f64.powf(delay as f64 / 1200.0)
                    };
                    let attack_sec = 2.0_f64.powf(attack as f64 / 1200.0);
                    let hold_sec = if hold <= -12000 {
                        0.0
                    } else {
                        2.0_f64.powf(hold as f64 / 1200.0)
                    };
                    let decay_sec = 2.0_f64.powf(decay as f64 / 1200.0);
                    let release_sec = 2.0_f64.powf(release as f64 / 1200.0);

                    // sustain is in centibels of attenuation
                    let sustain_level = 10.0_f64.powf(-(sustain as f64) / 200.0);

                    let adsr = Adsr {
                        delay: delay_sec,
                        attack: attack_sec.max(0.001),
                        hold: hold_sec,
                        decay: decay_sec.max(0.001),
                        sustain: sustain_level.clamp(0.0, 1.0),
                        release: release_sec.max(0.001),
                    };

                    return Some((id, overriding_root_key, adsr));
                }
            }
        }

        None
    }

    fn velocity_to_ratio(velocity: u8) -> f64 {
        velocity as f64 / 127.0
    }

    pub fn tick(&mut self, delta_sec: f64) {
        self.now_time += delta_sec;
    }

    pub fn finished(&self) -> bool {
        self.now_time >= self.end_time
    }

    fn midi_key_to_freq(key: u8) -> f64 {
        440.0 * 2f64.powf((key as f64 - 69.0) / 12.0)
    }
}

impl Drop for SoundSource {
    fn drop(&mut self) {
        for node in &self.nodes {
            let _ = node.disconnect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_key_to_freq() {
        assert_eq!(SoundSource::midi_key_to_freq(69), 440.0);
        assert_eq!(SoundSource::midi_key_to_freq(21), 27.5);
        assert_eq!(SoundSource::midi_key_to_freq(57), 220.0);
        assert_eq!(SoundSource::midi_key_to_freq(81), 880.0);
    }
}
