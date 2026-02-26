use crate::soundfont::SoundFont;
use wasm_bindgen::prelude::*;
use web_sys::{AudioNode, OscillatorType, BaseAudioContext, BiquadFilterType, AudioBuffer};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynthType {
    Analog,
    FM,
    Origin,
    SoundFont,
}

#[derive(Clone, Copy)]
pub struct Adsr {
    pub attack: f64,
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
        synth_type: SynthType,
        soundfont: Option<&SoundFont>,
    ) -> Result<SoundSource, JsValue> {
        match synth_type {
            SynthType::Analog => Self::new_analog(context, destination_target, key, velocity, start_time, end_time),
            SynthType::FM => Self::new_fm(context, destination_target, key, velocity, start_time, end_time),
            SynthType::Origin => Self::new_origin(context, destination_target, key, velocity, start_time, end_time),
            SynthType::SoundFont => {
                if let Some(sf) = soundfont {
                    Self::new_soundfont(context, destination_target, key, velocity, program, bank, start_time, end_time, sf)
                } else {
                    // Fallback to Analog if SoundFont data is missing
                    Self::new_analog(context, destination_target, key, velocity, start_time, end_time)
                }
            }
        }
    }

    fn new_analog(context: &BaseAudioContext, destination_target: &AudioNode, key: u8, velocity: u8, start_time: f64, end_time: f64) -> Result<SoundSource, JsValue> {
        let adsr = Adsr{
            attack: 0.01,
            decay: 0.3,
            sustain: 0.4,
            release: 0.8,
        };
        let freq = Self::midi_key_to_freq(key);
        let velocity = Self::velocity_to_ratio(velocity);
        let sus_begin = start_time + adsr.attack + adsr.decay;
        let end_time = end_time.max(sus_begin);

        // 1. VCO
        let vco = context.create_oscillator()?;
        vco.set_type(OscillatorType::Sawtooth);
        vco.frequency().set_value(freq as f32);

        // Pitch Envelope
        vco.frequency().set_value_at_time((freq * 1.015) as f32, start_time)?;
        vco.frequency().exponential_ramp_to_value_at_time(freq as f32, start_time + 0.08)?;

        // 2. VCF
        let vcf = context.create_biquad_filter()?;
        vcf.set_type(BiquadFilterType::Lowpass);
        vcf.q().set_value(1.0);

        let base_freq = freq * 1.5;
        let peak_freq = freq * 8.0 * velocity;

        let vcf_freq = vcf.frequency();
        vcf_freq.set_value_at_time(base_freq as f32, start_time)?;
        vcf_freq.linear_ramp_to_value_at_time(peak_freq as f32, start_time + adsr.attack)?;
        
        // Filter Sustain
        let vcf_sus = base_freq + (peak_freq - base_freq) * adsr.sustain * 0.5;
        let target_sus = if vcf_sus < 100.0 { 100.0 } else { vcf_sus };
        vcf_freq.exponential_ramp_to_value_at_time(target_sus as f32, sus_begin)?;
        vcf_freq.set_value_at_time(target_sus as f32, end_time)?;
        vcf_freq.exponential_ramp_to_value_at_time(base_freq as f32, end_time + adsr.release)?;

        // 3. VCA
        let vca = context.create_gain()?;
        let vca_gain = vca.gain();
        
        vca_gain.set_value_at_time(0.0, start_time)?;
        vca_gain.linear_ramp_to_value_at_time(1.0 * velocity as f32, start_time + adsr.attack)?;
        let vca_sus = (adsr.sustain * velocity).max(0.0001);
        vca_gain.exponential_ramp_to_value_at_time(vca_sus as f32, sus_begin)?;
        vca_gain.set_value_at_time(vca_sus as f32, end_time)?;
        vca_gain.exponential_ramp_to_value_at_time(0.0001, end_time + adsr.release)?;

        // Connection
        vco.connect_with_audio_node(&vcf)?;
        vcf.connect_with_audio_node(&vca)?;
        vca.connect_with_audio_node(destination_target)?;

        // play
        vco.start_with_when(start_time)?;
        vco.stop_with_when(end_time + adsr.release + 0.1)?;

        // Cleanup time
        let cleanup_time = end_time + adsr.release + 0.2;

        Ok(SoundSource {
            nodes: vec![vco.into(), vcf.into(), vca.into()],
            now_time: start_time,
            end_time: cleanup_time,
        })
    }

    fn new_fm(context: &BaseAudioContext, destination_target: &AudioNode, key: u8, velocity: u8, start_time: f64, end_time: f64) -> Result<SoundSource, JsValue> {
        let freq = Self::midi_key_to_freq(key);
        let velocity = Self::velocity_to_ratio(velocity);
        
        let adsr = Adsr{
            attack: 0.01,
            decay: 0.3,
            sustain: 0.4,
            release: 0.8,
        };

        let end_time = end_time.max(start_time + adsr.attack + adsr.decay);

        let carrier = context.create_oscillator()?;
        let modulator = context.create_oscillator()?;
        let amp_gain = context.create_gain()?;
        let modulator_gain = context.create_gain()?;

        // 2. 基本設定
        carrier.set_type(OscillatorType::Sine);
        modulator.set_type(OscillatorType::Sine);

        // ピアノらしい Ratio (C:M) = 1:1.001
        carrier.frequency().set_value(freq as f32);
        modulator.frequency().set_value((freq * 1.001) as f32);

        // 3. モジュレーターのエンベロープ（音色の変化）
        let index: f64 = 3.0; // FMの強さ
        modulator_gain.gain().set_value_at_time((freq * index * velocity) as f32, start_time)?;
        modulator_gain.gain().exponential_ramp_to_value_at_time(0.01 as f32, start_time + 0.3)?; 

        // 4. アンプのエンベロープ（音量の変化）
        amp_gain.gain().set_value_at_time(0.0, start_time)?;
        amp_gain.gain().linear_ramp_to_value_at_time((velocity) as f32, start_time + adsr.attack)?; // Attack
        amp_gain.gain().exponential_ramp_to_value_at_time((velocity * adsr.sustain) as f32, start_time + adsr.attack + adsr.decay)?;  // Decay
        amp_gain.gain().set_value_at_time((velocity * adsr.sustain) as f32, end_time)?;  // Sustain
        amp_gain.gain().exponential_ramp_to_value_at_time(0.0001 as f32, end_time + adsr.release)?;  // Release

        // 5. 接続
        modulator.connect_with_audio_node(&modulator_gain)?;
        modulator_gain.connect_with_audio_param(&carrier.frequency())?; 
        carrier.connect_with_audio_node(&amp_gain)?;
        amp_gain.connect_with_audio_node(destination_target)?;

        // 6. 再生開始
        modulator.start_with_when(start_time)?;
        carrier.start_with_when(start_time)?;
        modulator.stop_with_when(end_time + adsr.release + 0.1)?;
        carrier.stop_with_when(end_time + adsr.release + 0.1)?;

        let cleanup_time = end_time + adsr.release + 0.2;

        Ok(SoundSource {
            nodes: vec![carrier.into(), modulator.into(), amp_gain.into(), modulator_gain.into()],
            now_time: start_time,
            end_time: cleanup_time,
        })
    }

    fn new_origin(context: &BaseAudioContext, destination_target: &AudioNode, key: u8, velocity: u8, start_time: f64, end_time: f64) -> Result<SoundSource, JsValue> {
        let adsr = Adsr{
            attack: 0.01,
            decay: 0.2,
            sustain: 0.5,
            release: 1.0,
        };
        let freq = Self::midi_key_to_freq(key);
        let velocity = Self::velocity_to_ratio(velocity);
        let sus_begin = start_time + adsr.attack + adsr.decay;
        let end_time = end_time.max(sus_begin);

        // 1. VCO
        let vco = context.create_oscillator()?;
        vco.set_type(OscillatorType::Sawtooth);
        vco.frequency().set_value(freq as f32);

        // 2. VCF
        let vcf = context.create_biquad_filter()?;
        vcf.set_type(BiquadFilterType::Lowpass);
        vcf.frequency().set_value((freq * 4.0).min(10000.0) as f32);
        vcf.frequency().linear_ramp_to_value_at_time((freq * 0.5) as f32, end_time)?;

        // 3. VCA
        let vca = context.create_gain()?;
        let vca_gain = vca.gain();
        
        vca_gain.set_value_at_time(0.0, start_time)?;
        vca_gain.linear_ramp_to_value_at_time(velocity as f32, start_time + adsr.attack)?;
        vca_gain.linear_ramp_to_value_at_time((velocity * adsr.sustain) as f32, sus_begin)?;
        vca_gain.set_value_at_time((velocity * adsr.sustain) as f32, end_time)?;
        vca_gain.linear_ramp_to_value_at_time(0.0001, end_time + adsr.release)?;

        // Connection
        vco.connect_with_audio_node(&vcf)?;
        vcf.connect_with_audio_node(&vca)?;
        vca.connect_with_audio_node(destination_target)?;

        // play
        vco.start_with_when(start_time)?;
        vco.stop_with_when(end_time + adsr.release + 0.1)?;

        // Cleanup time
        let cleanup_time = end_time + adsr.release + 0.2;

        Ok(SoundSource {
            nodes: vec![vco.into(), vcf.into(), vca.into()],
            now_time: start_time,
            end_time: cleanup_time,
        })
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
        let adsr = Adsr{
            attack: 0.01,
            decay: 0.2,
            sustain: 0.8,
            release: 0.5,
        };
        let _freq = Self::midi_key_to_freq(key);
        let vel_ratio = Self::velocity_to_ratio(velocity);
        let sus_begin = start_time + adsr.attack + adsr.decay;
        let end_time = end_time.max(sus_begin);

        // 使用するサンプルのインデックスを探す
        let (sample_idx, overriding_root_key) = Self::find_sample_index(soundfont, bank, program, key, velocity).unwrap_or((0, None));

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
            let sample_len = end_idx.saturating_sub(start_idx);
            
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

        // 2. VCA (Volume Envelope)
        let vca = context.create_gain()?;
        let vca_gain = vca.gain();
        
        vca_gain.set_value_at_time(0.0, start_time)?;
        vca_gain.linear_ramp_to_value_at_time(vel_ratio as f32, start_time + adsr.attack)?;
        let vca_sus = (adsr.sustain * vel_ratio).max(0.0001);
        vca_gain.linear_ramp_to_value_at_time(vca_sus as f32, sus_begin)?;
        vca_gain.set_value_at_time(vca_sus as f32, end_time)?;
        vca_gain.linear_ramp_to_value_at_time(0.0001, end_time + adsr.release)?;

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
    fn find_sample_index(soundfont: &SoundFont, bank: u16, program: u8, key: u8, velocity: u8) -> Option<(usize, Option<u8>)> {
        // 1. 該当のプリセットを検索
        let preset_idx = soundfont.preset_headers.iter()
            .position(|p| p.preset == program as u16 && p.bank == bank)
            // 該当がなければ bank 変えずに program のみ、あるいは bank 0 にフォールバックなどを検討（ここでは柔軟にヒットさせる）
            .or_else(|| soundfont.preset_headers.iter().position(|p| p.preset == program as u16))
            .or_else(|| soundfont.preset_headers.iter().position(|p| p.preset == 0 && p.bank == 0))?;
            
        let pbag_start = soundfont.preset_headers[preset_idx].preset_bag_ndx as usize;
        let pbag_end = soundfont.preset_headers.get(preset_idx + 1)
            .map(|p| p.preset_bag_ndx as usize)
            .unwrap_or(soundfont.preset_bags.len());

        let mut matched_instrument_id = None;

        // 2. プリセットから一致するゾーン（インストゥルメント）を検索
        for b in pbag_start..pbag_end {
            let gen_start = soundfont.preset_bags[b].gen_ndx as usize;
            let gen_end = soundfont.preset_bags.get(b + 1)
                .map(|bg| bg.gen_ndx as usize)
                .unwrap_or(soundfont.preset_generators.len());

            let mut key_in_range = true;
            let mut vel_in_range = true;
            let mut inst_id = None;

            for g in gen_start..gen_end {
                if let Some(generator) = soundfont.preset_generators.get(g) {
                    match generator.gen_oper {
                        43 => { // keyRange
                            let lo = (generator.gen_amount & 0xFF) as u8;
                            let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                            if key < lo || key > hi { key_in_range = false; }
                        }
                        44 => { // velRange
                            let lo = (generator.gen_amount & 0xFF) as u8;
                            let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                            if velocity < lo || velocity > hi { vel_in_range = false; }
                        }
                        41 => { // instrument
                            inst_id = Some(generator.gen_amount as usize);
                        }
                        _ => {}
                    }
                }
            }

            // 条件に一致かつインストゥルメントIDが見つかった場合
            if key_in_range && vel_in_range {
                if let Some(id) = inst_id {
                    matched_instrument_id = Some(id);
                    break;
                }
            }
        }

        let inst_id = matched_instrument_id?;

        // 3. インストゥルメントから一致するサンプルを検索
        let ibag_start = soundfont.instruments.get(inst_id)?.inst_bag_ndx as usize;
        let ibag_end = soundfont.instruments.get(inst_id + 1)
            .map(|i| i.inst_bag_ndx as usize)
            .unwrap_or(soundfont.instrument_bags.len());

        for b in ibag_start..ibag_end {
            let gen_start = soundfont.instrument_bags.get(b)?.inst_gen_ndx as usize;
            let gen_end = soundfont.instrument_bags.get(b + 1)
                .map(|bg| bg.inst_gen_ndx as usize)
                .unwrap_or(soundfont.instrument_generators.len());

            let mut key_in_range = true;
            let mut vel_in_range = true;
            let mut sample_id = None;
            let mut overriding_root_key = None;

            for g in gen_start..gen_end {
                if let Some(generator) = soundfont.instrument_generators.get(g) {
                    match generator.gen_oper {
                        43 => { // keyRange
                            let lo = (generator.gen_amount & 0xFF) as u8;
                            let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                            if key < lo || key > hi { key_in_range = false; }
                        }
                        44 => { // velRange
                            let lo = (generator.gen_amount & 0xFF) as u8;
                            let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                            if velocity < lo || velocity > hi { vel_in_range = false; }
                        }
                        53 => { // sampleID
                            sample_id = Some(generator.gen_amount as usize);
                        }
                        58 => { // overridingRootKey
                            overriding_root_key = Some(generator.gen_amount as u8);
                        }
                        _ => {}
                    }
                }
            }

            if key_in_range && vel_in_range {
                if let Some(id) = sample_id {
                    return Some((id, overriding_root_key));
                }
            }
        }

        None
    }

    fn velocity_to_ratio(velocity: u8) -> f64{
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