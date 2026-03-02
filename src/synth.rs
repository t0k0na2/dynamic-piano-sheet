use crate::note::PitchBendEvent;
use crate::note::VolumeEvent;
use crate::soundfont::{GeneratorOperator, SoundFont};
use wasm_bindgen::prelude::*;
use web_sys::{AudioNode, BaseAudioContext};

#[derive(Clone, Copy, Debug)]
pub struct SampleOffsets {
    pub start: i32,
    pub end: i32,
    pub start_loop: i32,
    pub end_loop: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct LfoParams {
    pub delay: f64,
    pub freq: f32,
    pub to_pitch: f32,
    pub to_filter: f32,
    pub to_volume: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ModEnvParams {
    pub adsr: Adsr,
    pub to_pitch: f32,
    pub to_filter: f32,
}

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
        channel_volumes: &[VolumeEvent],
        pitch_bends: &[PitchBendEvent],
        pitch_bend_sensitivity: f32,
        channel: u8,
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
                channel_volumes,
                pitch_bends,
                pitch_bend_sensitivity,
                channel,
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
        channel_volumes: &[VolumeEvent],
        pitch_bends: &[PitchBendEvent],
        pitch_bend_sensitivity: f32,
        channel: u8,
        program: u8,
        mut bank: u16,
        start_time: f64,
        end_time: f64,
        soundfont: &SoundFont,
    ) -> Result<SoundSource, JsValue> {
        // MIDI channel 10 はパーカッション用のため、Bankを128に強制する
        bank = if channel == 10 { 128 } else { bank };
        // 使用するサンプルのインデックスを探す
        let (
            sample_idx,
            overriding_root_key,
            adsr,
            mod_env,
            filter_fc,
            filter_q,
            sample_modes,
            sample_offsets,
            coarse_tune,
            fine_tune,
            mod_lfo,
            vib_lfo,
            scale_tuning,
            _vol_factor,
        ) = match Self::find_sample_index(soundfont, bank, program, key, velocity) {
            Some(params) => params,
            None => {
                // サンプルが見つからない場合は無音のSoundSourceを返す
                crate::log!("Sample not found for key: {}", key);
                return Ok(SoundSource {
                    nodes: vec![],
                    now_time: start_time,
                    end_time: start_time,
                });
            }
        };

        let _freq = Self::midi_key_to_freq(key);
        let vel_ratio = Self::velocity_to_ratio(velocity); // * vol_factor as f64;

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

        let pitch_correction = if let Some(h) = shdr {
            h.pitch_correction as f32
        } else {
            0.0
        };

        // overridingRootKey が指定されている場合は優先
        if let Some(root_key) = overriding_root_key {
            // SF2の仕様では0~127が有効とされている
            if root_key <= 127 {
                original_pitch = root_key as f32;
            }
        }

        let key_pitch = key as f32;
        // ピッチの計算
        // Scale Tuning が指定されていれば、それがピッチのスケーリングに使われる（デフォルト 100% = 1.0）
        // パーカッション(Channel 10)などで Scale Tuning が0の場合はキーによるピッチ変化がおきない
        let scale_tuning_ratio = scale_tuning / 100.0;
        let exponent = ((key_pitch - original_pitch) * scale_tuning_ratio + coarse_tune as f32)
            / 12.0
            + (pitch_correction + fine_tune as f32) / 1200.0;
        let playback_rate = 2.0_f32.powf(exponent);

        // 動的にAudioBufferを生成する
        let audio_buffer = if let Some(h) = shdr {
            let start_idx = (h.start as i64 + sample_offsets.start as i64).max(0) as usize;
            let end_idx = (h.end as i64 + sample_offsets.end as i64).max(0) as usize;

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

        // --- Pitch Bend 処理 ---
        let pb_range_semitones = pitch_bend_sensitivity; // RPN から取得した値を使用
        let calc_pb_rate = |bend: i16| -> f32 {
            let bend_norm = bend as f32 / 8192.0;
            let bend_semitones = bend_norm * pb_range_semitones;
            let bend_rate = 2.0_f32.powf(bend_semitones / 12.0);
            playback_rate * bend_rate
        };

        let pb_param = source_node.playback_rate();
        let init_pb = pitch_bends.first().map(|v| v.bend).unwrap_or(0);

        // 常にset_value_at_timeで初期ピッチを設定する
        pb_param.set_value_at_time(calc_pb_rate(init_pb), start_time)?;

        let pb_on_time = pitch_bends.first().map(|v| v.time).unwrap_or(0.0);
        for event in pitch_bends.iter().skip(1) {
            let event_time = start_time + (event.time - pb_on_time);
            if event_time > start_time {
                pb_param.set_target_at_time(calc_pb_rate(event.bend), event_time, 0.01)?;
            }
        }

        // -- LFO Setup --
        let mut lfo_nodes = vec![];
        let source_detune = source_node.detune();

        let mut mod_lfo_osc = None;
        if mod_lfo.to_pitch != 0.0 || mod_lfo.to_filter != 0.0 || mod_lfo.to_volume != 0.0 {
            let osc = context.create_oscillator()?;
            osc.set_type(web_sys::OscillatorType::Triangle);
            osc.frequency().set_value(mod_lfo.freq);
            mod_lfo_osc = Some(osc.clone());
            lfo_nodes.push(osc.into());
        }

        let mut vib_lfo_osc = None;
        if vib_lfo.to_pitch != 0.0 {
            let osc = context.create_oscillator()?;
            osc.set_type(web_sys::OscillatorType::Triangle);
            osc.frequency().set_value(vib_lfo.freq);
            vib_lfo_osc = Some(osc.clone());
            lfo_nodes.push(osc.into());
        }

        if let Some(osc) = &mod_lfo_osc {
            if mod_lfo.to_pitch != 0.0 {
                let p_gain = context.create_gain()?;
                p_gain.gain().set_value(mod_lfo.to_pitch);
                osc.connect_with_audio_node(&p_gain)?;
                p_gain.connect_with_audio_param(&source_detune)?;
                lfo_nodes.push(p_gain.into());
            }
        }

        if let Some(osc) = &vib_lfo_osc {
            if vib_lfo.to_pitch != 0.0 {
                let p_gain = context.create_gain()?;
                p_gain.gain().set_value(vib_lfo.to_pitch);
                osc.connect_with_audio_node(&p_gain)?;
                p_gain.connect_with_audio_param(&source_detune)?;
                lfo_nodes.push(p_gain.into());
            }
        }

        // sf2のサンプルヘッダーからループポイント等の情報を取得して設定
        if let Some(h) = shdr {
            // sampleModes: 0 (no loop), 1 (loop continuously), 2 (no loop), 3 (loop for duration of key depression)
            let loop_enabled = sample_modes == 1 || sample_modes == 3;
            source_node.set_loop(loop_enabled);

            if loop_enabled {
                let effective_start =
                    (h.start as i64 + sample_offsets.start as i64).max(0) as usize;
                let loop_start_idx =
                    (h.start_loop as i64 + sample_offsets.start_loop as i64).max(0) as usize;
                let loop_end_idx =
                    (h.end_loop as i64 + sample_offsets.end_loop as i64).max(0) as usize;

                // 動的生成の場合、取り出したAudioBufferサイズに合わせた相対位置に直す
                let rel_loop_start =
                    loop_start_idx.saturating_sub(effective_start) as f64 / sample_rate as f64;
                let rel_loop_end =
                    loop_end_idx.saturating_sub(effective_start) as f64 / sample_rate as f64;
                source_node.set_loop_start(rel_loop_start.max(0.0));
                source_node.set_loop_end(rel_loop_end.max(0.0));
            }
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
        vca_gain.exponential_ramp_to_value_at_time((vel_ratio as f32).max(0.0001), current_time)?;

        if adsr.hold > 0.0 {
            current_time += adsr.hold;
            vca_gain.set_value_at_time((vel_ratio as f32).max(0.0001), current_time)?;
        }

        current_time += adsr.decay;
        let vca_sus = (adsr.sustain * vel_ratio).max(0.0001);
        if end_time > current_time {
            vca_gain
                .exponential_ramp_to_value_at_time((vca_sus as f32).max(0.0001), current_time)?;
            vca_gain.exponential_ramp_to_value_at_time((vca_sus as f32).max(0.0001), end_time)?;
        } else {
            // end_time が current_time より小さい場合は、end_timeまでのターゲットボリュームを計算して反映する
            // Web Audio APIの exponential_ramp_to_value と同じ計算式を使用する
            // V(t) = V0 * (V1 / V0) ^ ((t - T0) / (T1 - T0))
            let t0 = current_time - adsr.decay;
            let v0 = (vel_ratio as f64).max(0.0001);
            let target_volume = if adsr.decay > 0.0 && end_time > t0 {
                v0 * (vca_sus / v0).powf((end_time - t0) / adsr.decay)
            } else {
                v0 // AttackやHoldフェーズの途中で離鍵された場合はピーク音量(v0)とする
            };
            vca_gain
                .exponential_ramp_to_value_at_time((target_volume as f32).max(0.0001), end_time)?;
        }
        vca_gain.exponential_ramp_to_value_at_time(0.0001, end_time + adsr.release)?;

        // Filter (BiquadFilterNode - Lowpass) を構築して initialFilterFc, initialFilterQ を設定
        let filter = context.create_biquad_filter()?;
        filter.set_type(web_sys::BiquadFilterType::Lowpass);

        // initialFilterFc: セント単位 (0 cents = 8.176 Hz) => Hz = 440 * 2^((cents - 6900) / 1200)
        let freq_hz = 440.0 * 2.0_f32.powf((filter_fc - 6900.0) / 1200.0);
        let max_freq = context.sample_rate() / 2.0;
        let clamped_freq = freq_hz.min(max_freq).max(0.0);
        filter.frequency().set_value(clamped_freq);

        // initialFilterQ: セントベル(cB)単位。Web Audio APIのQ値(Lowpass用)はデシベル(dB)なので 10.0 で割る
        let q_db = filter_q / 10.0;
        filter.q().set_value(q_db);

        // -- Apply ModEnv to Pitch & FilterFc --
        let apply_env = |param: &web_sys::AudioParam, amount: f32| -> Result<(), JsValue> {
            if amount == 0.0 {
                return Ok(());
            }
            let mut current_time = start_time;
            param.set_value_at_time(0.0, current_time)?;

            if mod_env.adsr.delay > 0.0 {
                current_time += mod_env.adsr.delay;
                param.set_value_at_time(0.0, current_time)?;
            }

            current_time += mod_env.adsr.attack;
            param.linear_ramp_to_value_at_time(amount, current_time)?;

            if mod_env.adsr.hold > 0.0 {
                current_time += mod_env.adsr.hold;
                param.set_value_at_time(amount, current_time)?;
            }

            current_time += mod_env.adsr.decay;
            let sus_val = amount * mod_env.adsr.sustain as f32;
            param.linear_ramp_to_value_at_time(sus_val, current_time)?;

            if end_time > current_time {
                param.linear_ramp_to_value_at_time(sus_val, end_time)?;
            }

            param.linear_ramp_to_value_at_time(0.0, end_time + mod_env.adsr.release)?;
            Ok(())
        };

        apply_env(&source_detune, mod_env.to_pitch)?;
        apply_env(&filter.detune(), mod_env.to_filter)?;

        if let Some(osc) = &mod_lfo_osc {
            if mod_lfo.to_filter != 0.0 {
                let f_gain = context.create_gain()?;
                f_gain.gain().set_value(mod_lfo.to_filter);
                osc.connect_with_audio_node(&f_gain)?;
                f_gain.connect_with_audio_param(&filter.detune())?;
                lfo_nodes.push(f_gain.into());
            }
        }

        // Connection
        source_node.connect_with_audio_node(&filter)?;
        filter.connect_with_audio_node(&vca)?;

        let tremor_depth = 1.0 - 10.0_f32.powf(-mod_lfo.to_volume / 200.0);
        let mut tremolo_vca_node = None;
        if let Some(osc) = &mod_lfo_osc {
            if mod_lfo.to_volume > 0.0 && tremor_depth > 0.0 {
                let vca_lfo_gain = context.create_gain()?;
                vca_lfo_gain.gain().set_value(tremor_depth);

                let tremolo_vca = context.create_gain()?;
                tremolo_vca.gain().set_value(1.0);

                osc.connect_with_audio_node(&vca_lfo_gain)?;
                vca_lfo_gain.connect_with_audio_param(&tremolo_vca.gain())?;

                vca.connect_with_audio_node(&tremolo_vca)?;

                lfo_nodes.push(vca_lfo_gain.into());

                let tremolo_vca_audio_node: web_sys::AudioNode = tremolo_vca.into();
                lfo_nodes.push(tremolo_vca_audio_node.clone());
                tremolo_vca_node = Some(tremolo_vca_audio_node);
            }
        }

        let ch_vol_node = context.create_gain()?;
        let init_ch_vol = channel_volumes.first().map(|v| v.volume).unwrap_or(100);
        ch_vol_node
            .gain()
            .set_value(Self::velocity_to_ratio(init_ch_vol) as f32);

        let on_time = channel_volumes.first().map(|v| v.time).unwrap_or(0.0);
        for event in channel_volumes.iter().skip(1) {
            let event_time = start_time + (event.time - on_time);
            if event_time > start_time {
                ch_vol_node.gain().set_target_at_time(
                    Self::velocity_to_ratio(event.volume) as f32,
                    event_time,
                    0.01,
                )?;
            }
        }

        if let Some(t_vca) = tremolo_vca_node {
            t_vca.connect_with_audio_node(&ch_vol_node)?;
        } else {
            vca.connect_with_audio_node(&ch_vol_node)?;
        }
        ch_vol_node.connect_with_audio_node(destination_target)?;

        // Play
        // AudioBufferを切り出しているのでオフセットを0にする
        if let Some(_) = shdr {
            source_node.start_with_when(start_time)?;
        } else {
            source_node.start_with_when(start_time)?;
        }

        #[allow(deprecated)]
        source_node.stop_with_when(end_time + adsr.release + 0.1)?;

        if let Some(osc) = &mod_lfo_osc {
            osc.start_with_when(start_time + mod_lfo.delay)?;
            #[allow(deprecated)]
            osc.stop_with_when(end_time + adsr.release + 0.1)?;
        }
        if let Some(osc) = &vib_lfo_osc {
            osc.start_with_when(start_time + vib_lfo.delay)?;
            #[allow(deprecated)]
            osc.stop_with_when(end_time + adsr.release + 0.1)?;
        }

        let cleanup_time = end_time + adsr.release + 0.2;

        let mut final_nodes = vec![
            source_node.into(),
            filter.into(),
            vca.into(),
            ch_vol_node.into(),
        ];
        final_nodes.extend(lfo_nodes);

        Ok(SoundSource {
            nodes: final_nodes,
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
    ) -> Option<(
        usize,
        Option<u8>,
        Adsr,
        ModEnvParams,
        f32,
        f32,
        u16,
        SampleOffsets,
        f32,
        f32,
        LfoParams,
        LfoParams,
        f32,
        f32,
    )> {
        // 1. 該当のプリセットを検索
        let preset_idx = soundfont
            .preset_headers
            .iter()
            .position(|p| p.preset == program as u16 && p.bank == bank)
            // 該当がなければ別のBank/Programにフォールバック
            .or_else(|| {
                if bank == 128 {
                    // パーカッションで該当キットがない場合は標準ドラムキット(Bank 128, Preset 0)にフォールバック
                    soundfont
                        .preset_headers
                        .iter()
                        .position(|p| p.preset == 0 && p.bank == 128)
                } else {
                    // 通常楽器の場合は他のBankで同じProgramを探す(ただしBank 128のパーカッション以外)
                    soundfont
                        .preset_headers
                        .iter()
                        .position(|p| p.preset == program as u16 && p.bank != 128)
                }
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

        // 2. プリセットから一致するゾーンを検索し、その中でインストゥルメントを検索
        let mut p_global_gens = [None; 60];

        for b in pbag_start..pbag_end {
            let gen_start = soundfont.preset_bags[b].gen_ndx as usize;
            let gen_end = soundfont
                .preset_bags
                .get(b + 1)
                .map(|bg| bg.gen_ndx as usize)
                .unwrap_or(soundfont.preset_generators.len());

            let mut key_in_range_p = true;
            let mut vel_in_range_p = true;
            let mut inst_id = None;
            let mut preset_local_gens = [None; 60];

            for g in gen_start..gen_end {
                if let Some(generator) = soundfont.preset_generators.get(g) {
                    if (generator.gen_oper as usize) < 60 {
                        preset_local_gens[generator.gen_oper as usize] = Some(generator.gen_amount);
                    }
                    if let Ok(op) = std::convert::TryFrom::try_from(generator.gen_oper) {
                        match op {
                            GeneratorOperator::KeyRange => {
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if key < lo || key > hi {
                                    key_in_range_p = false;
                                }
                            }
                            GeneratorOperator::VelRange => {
                                let lo = (generator.gen_amount & 0xFF) as u8;
                                let hi = ((generator.gen_amount >> 8) & 0xFF) as u8;
                                if velocity < lo || velocity > hi {
                                    vel_in_range_p = false;
                                }
                            }
                            GeneratorOperator::Instrument => {
                                inst_id = Some(generator.gen_amount as usize);
                            }
                            _ => {}
                        }
                    }
                }
            }

            if inst_id.is_none() && b == pbag_start {
                p_global_gens = preset_local_gens;
                continue;
            }

            if key_in_range_p && vel_in_range_p {
                if let Some(id) = inst_id {
                    let preset_gens = preset_local_gens;

                    // 3. インストゥルメントから一致するサンプルを検索
                    if let Some(instrument) = soundfont.instruments.get(id) {
                        let ibag_start = instrument.inst_bag_ndx as usize;
                        let ibag_end = soundfont
                            .instruments
                            .get(id + 1)
                            .map(|i| i.inst_bag_ndx as usize)
                            .unwrap_or(soundfont.instrument_bags.len());

                        let mut i_global_gens = [None; 60];

                        for ib in ibag_start..ibag_end {
                            if let Some(ibag) = soundfont.instrument_bags.get(ib) {
                                let igen_start = ibag.inst_gen_ndx as usize;
                                let igen_end = soundfont
                                    .instrument_bags
                                    .get(ib + 1)
                                    .map(|bg| bg.inst_gen_ndx as usize)
                                    .unwrap_or(soundfont.instrument_generators.len());

                                let mut key_in_range_i = true;
                                let mut vel_in_range_i = true;
                                let mut sample_id = None;
                                let mut inst_local_gens = [None; 60];

                                for ig in igen_start..igen_end {
                                    if let Some(generator) = soundfont.instrument_generators.get(ig)
                                    {
                                        if (generator.gen_oper as usize) < 60 {
                                            inst_local_gens[generator.gen_oper as usize] =
                                                Some(generator.gen_amount);
                                        }
                                        if let Ok(op) =
                                            std::convert::TryFrom::try_from(generator.gen_oper)
                                        {
                                            match op {
                                                GeneratorOperator::KeyRange => {
                                                    let lo = (generator.gen_amount & 0xFF) as u8;
                                                    let hi =
                                                        ((generator.gen_amount >> 8) & 0xFF) as u8;
                                                    if key < lo || key > hi {
                                                        key_in_range_i = false;
                                                    }
                                                }
                                                GeneratorOperator::VelRange => {
                                                    let lo = (generator.gen_amount & 0xFF) as u8;
                                                    let hi =
                                                        ((generator.gen_amount >> 8) & 0xFF) as u8;
                                                    if velocity < lo || velocity > hi {
                                                        vel_in_range_i = false;
                                                    }
                                                }
                                                GeneratorOperator::SampleID => {
                                                    sample_id = Some(generator.gen_amount as usize);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }

                                if sample_id.is_none() && ib == ibag_start {
                                    i_global_gens = inst_local_gens;
                                    continue;
                                }

                                if key_in_range_i && vel_in_range_i {
                                    if let Some(sid) = sample_id {
                                        let get_gen =
                                            |oper: GeneratorOperator, default: i16| -> i16 {
                                                let op_idx = oper as usize;
                                                let inst_val = inst_local_gens[op_idx]
                                                    .or(i_global_gens[op_idx])
                                                    .unwrap_or(default);
                                                let preset_val = preset_gens[op_idx]
                                                    .or(p_global_gens[op_idx])
                                                    .unwrap_or(0);
                                                inst_val.saturating_add(preset_val)
                                            };

                                        let delay = get_gen(GeneratorOperator::DelayVolEnv, -12000);
                                        let attack =
                                            get_gen(GeneratorOperator::AttackVolEnv, -12000);
                                        let hold = get_gen(GeneratorOperator::HoldVolEnv, -12000);
                                        let decay = get_gen(GeneratorOperator::DecayVolEnv, -12000);
                                        let sustain = get_gen(GeneratorOperator::SustainVolEnv, 0);
                                        let release =
                                            get_gen(GeneratorOperator::ReleaseVolEnv, -12000);

                                        let keynum_to_vol_hold =
                                            get_gen(GeneratorOperator::KeynumToVolEnvHold, 0);
                                        let keynum_to_vol_decay =
                                            get_gen(GeneratorOperator::KeynumToVolEnvDecay, 0);

                                        let key_diff = 60 - (key as i16);
                                        let eff_vol_hold = hold.saturating_add(
                                            keynum_to_vol_hold.saturating_mul(key_diff),
                                        );
                                        let eff_vol_decay = decay.saturating_add(
                                            keynum_to_vol_decay.saturating_mul(key_diff),
                                        );

                                        let initial_filter_fc =
                                            get_gen(GeneratorOperator::InitialFilterFc, 13500);
                                        let initial_filter_q =
                                            get_gen(GeneratorOperator::InitialFilterQ, 0);
                                        let sample_modes =
                                            get_gen(GeneratorOperator::SampleModes, 0);

                                        let start_addrs_offset =
                                            get_gen(GeneratorOperator::StartAddrsOffset, 0);
                                        let end_addrs_offset =
                                            get_gen(GeneratorOperator::EndAddrsOffset, 0);
                                        let startloop_addrs_offset =
                                            get_gen(GeneratorOperator::StartloopAddrsOffset, 0);
                                        let endloop_addrs_offset =
                                            get_gen(GeneratorOperator::EndloopAddrsOffset, 0);
                                        let start_addrs_coarse_offset =
                                            get_gen(GeneratorOperator::StartAddrsCoarseOffset, 0);
                                        let end_addrs_coarse_offset =
                                            get_gen(GeneratorOperator::EndAddrsCoarseOffset, 0);
                                        let startloop_addrs_coarse_offset = get_gen(
                                            GeneratorOperator::StartloopAddrsCoarseOffset,
                                            0,
                                        );
                                        let endloop_addrs_coarse_offset =
                                            get_gen(GeneratorOperator::EndloopAddrsCoarseOffset, 0);

                                        let sample_offsets = SampleOffsets {
                                            start: start_addrs_offset as i32
                                                + (start_addrs_coarse_offset as i32 * 32768),
                                            end: end_addrs_offset as i32
                                                + (end_addrs_coarse_offset as i32 * 32768),
                                            start_loop: startloop_addrs_offset as i32
                                                + (startloop_addrs_coarse_offset as i32 * 32768),
                                            end_loop: endloop_addrs_offset as i32
                                                + (endloop_addrs_coarse_offset as i32 * 32768),
                                        };

                                        let delay_sec = if delay <= -12000 {
                                            0.0
                                        } else {
                                            2.0_f64.powf(delay as f64 / 1200.0)
                                        };
                                        let attack_sec = 2.0_f64.powf(attack as f64 / 1200.0);
                                        let hold_sec = if eff_vol_hold <= -12000 {
                                            0.0
                                        } else {
                                            2.0_f64.powf(eff_vol_hold as f64 / 1200.0)
                                        };
                                        let decay_sec = 2.0_f64.powf(eff_vol_decay as f64 / 1200.0);
                                        let release_sec = 2.0_f64.powf(release as f64 / 1200.0);
                                        let sustain_level =
                                            10.0_f64.powf(-(sustain as f64) / 200.0);

                                        let coarse_tune = get_gen(GeneratorOperator::CoarseTune, 0);
                                        let fine_tune = get_gen(GeneratorOperator::FineTune, 0);

                                        let del_mod =
                                            get_gen(GeneratorOperator::DelayModEnv, -12000);
                                        let att_mod =
                                            get_gen(GeneratorOperator::AttackModEnv, -12000);
                                        let hld_mod =
                                            get_gen(GeneratorOperator::HoldModEnv, -12000);
                                        let dec_mod =
                                            get_gen(GeneratorOperator::DecayModEnv, -12000);
                                        let sus_mod = get_gen(GeneratorOperator::SustainModEnv, 0);
                                        let rel_mod =
                                            get_gen(GeneratorOperator::ReleaseModEnv, -12000);

                                        let keynum_to_mod_hold =
                                            get_gen(GeneratorOperator::KeynumToModEnvHold, 0);
                                        let keynum_to_mod_decay =
                                            get_gen(GeneratorOperator::KeynumToModEnvDecay, 0);
                                        let mod_env_to_pitch =
                                            get_gen(GeneratorOperator::ModEnvToPitch, 0);
                                        let mod_env_to_filter_fc =
                                            get_gen(GeneratorOperator::ModEnvToFilterFc, 0);

                                        let eff_hld_mod = hld_mod.saturating_add(
                                            keynum_to_mod_hold.saturating_mul(key_diff),
                                        );
                                        let eff_dec_mod = dec_mod.saturating_add(
                                            keynum_to_mod_decay.saturating_mul(key_diff),
                                        );

                                        let mod_env = ModEnvParams {
                                            adsr: Adsr {
                                                delay: if del_mod <= -12000 {
                                                    0.0
                                                } else {
                                                    2.0_f64.powf(del_mod as f64 / 1200.0)
                                                },
                                                attack: 2.0_f64
                                                    .powf(att_mod as f64 / 1200.0)
                                                    .max(0.001),
                                                hold: if eff_hld_mod <= -12000 {
                                                    0.0
                                                } else {
                                                    2.0_f64.powf(eff_hld_mod as f64 / 1200.0)
                                                },
                                                decay: 2.0_f64
                                                    .powf(eff_dec_mod as f64 / 1200.0)
                                                    .max(0.001),
                                                sustain: (1.0 - (sus_mod as f64 / 1000.0))
                                                    .clamp(0.0, 1.0),
                                                release: 2.0_f64
                                                    .powf(rel_mod as f64 / 1200.0)
                                                    .max(0.001),
                                            },
                                            to_pitch: mod_env_to_pitch as f32,
                                            to_filter: mod_env_to_filter_fc as f32,
                                        };

                                        let mod_lfo_delay =
                                            get_gen(GeneratorOperator::DelayModLFO, -12000);
                                        let mod_lfo_freq =
                                            get_gen(GeneratorOperator::FreqModLFO, 0);
                                        let mod_lfo_to_pitch =
                                            get_gen(GeneratorOperator::ModLfoToPitch, 0);
                                        let mod_lfo_to_filter =
                                            get_gen(GeneratorOperator::ModLfoToFilterFc, 0);
                                        let mod_lfo_to_volume =
                                            get_gen(GeneratorOperator::ModLfoToVolume, 0);

                                        let vib_lfo_delay =
                                            get_gen(GeneratorOperator::DelayVibLFO, -12000);
                                        let vib_lfo_freq =
                                            get_gen(GeneratorOperator::FreqVibLFO, 0);
                                        let vib_lfo_to_pitch =
                                            get_gen(GeneratorOperator::VibLfoToPitch, 0);

                                        let calc_lfo_delay = |delay_cents: i16| -> f64 {
                                            if delay_cents <= -12000 {
                                                0.0
                                            } else {
                                                2.0_f64.powf(delay_cents as f64 / 1200.0)
                                            }
                                        };

                                        let calc_lfo_freq = |freq_cents: i16| -> f32 {
                                            8.176 * 2.0_f32.powf(freq_cents as f32 / 1200.0)
                                        };

                                        let mod_lfo = LfoParams {
                                            delay: calc_lfo_delay(mod_lfo_delay),
                                            freq: calc_lfo_freq(mod_lfo_freq),
                                            to_pitch: mod_lfo_to_pitch as f32,
                                            to_filter: mod_lfo_to_filter as f32,
                                            to_volume: mod_lfo_to_volume as f32,
                                        };

                                        let vib_lfo = LfoParams {
                                            delay: calc_lfo_delay(vib_lfo_delay),
                                            freq: calc_lfo_freq(vib_lfo_freq),
                                            to_pitch: vib_lfo_to_pitch as f32,
                                            to_filter: 0.0,
                                            to_volume: 0.0,
                                        };

                                        let adsr = Adsr {
                                            delay: delay_sec,
                                            attack: attack_sec.max(0.001),
                                            hold: hold_sec,
                                            decay: decay_sec.max(0.001),
                                            sustain: sustain_level.clamp(0.0, 1.0),
                                            release: release_sec.max(0.001),
                                        };

                                        let ork = inst_local_gens
                                            [GeneratorOperator::OverridingRootKey as usize]
                                            .or(i_global_gens
                                                [GeneratorOperator::OverridingRootKey as usize])
                                            .unwrap_or(-1);
                                        let calculated_overriding_root_key =
                                            if ork >= 0 && ork <= 127 {
                                                Some(ork as u8)
                                            } else {
                                                None
                                            };

                                        let scale_tuning =
                                            get_gen(GeneratorOperator::ScaleTuning, 100) as f32;

                                        let initial_attenuation =
                                            get_gen(GeneratorOperator::InitialAttenuation, 0);
                                        let vol_factor =
                                            10.0_f32.powf(-(initial_attenuation as f32) / 200.0);

                                        return Some((
                                            sid,
                                            calculated_overriding_root_key,
                                            adsr,
                                            mod_env,
                                            initial_filter_fc as f32,
                                            initial_filter_q as f32,
                                            sample_modes as u16,
                                            sample_offsets,
                                            coarse_tune as f32,
                                            fine_tune as f32,
                                            mod_lfo,
                                            vib_lfo,
                                            scale_tuning,
                                            vol_factor,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn velocity_to_ratio(velocity: u8) -> f64 {
        (velocity as f32 / 127.0).powi(2) as f64
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

    #[test]
    fn test_find_percussion_samples() {
        let sf2_data = std::fs::read("test.sf2");
        if let Ok(data) = sf2_data {
            use crate::soundfont::SoundFont;
            let sf = SoundFont::parse(&data).expect("Failed to parse test.sf2");

            let bank = 128;
            let program = 16;
            let velocity = 100;

            println!("Testing drum kit Bank: {}, Program: {}", bank, program);

            for key in 35..=81 {
                let result = SoundSource::find_sample_index(&sf, bank, program, key, velocity);
                assert!(
                    result.is_some(),
                    "Failed to find sample for percussion key {} in Bank {} Program {}",
                    key,
                    bank,
                    program
                );
                if let Some((idx, _, _, _, _, _, _, _, _, _, _, _, _, _)) = result {
                    println!("Key: {:>2} -> Sample Index: {}", key, idx);
                }
            }
        } else {
            println!("test.sf2 not found. Skipping percussion test.");
        }
    }

    #[test]
    fn test_find_samples() {
        let sf2_data = std::fs::read("test.sf2");
        if let Ok(data) = sf2_data {
            use crate::soundfont::SoundFont;
            let sf = SoundFont::parse(&data).expect("Failed to parse test.sf2");

            let bank = 121;
            let program = 1;
            let velocity = 100;

            println!("Testing drum kit Bank: {}, Program: {}", bank, program);

            for key in 35..=81 {
                let result = SoundSource::find_sample_index(&sf, bank, program, key, velocity);
                assert!(
                    result.is_some(),
                    "Failed to find sample for percussion key {} in Bank {} Program {}",
                    key,
                    bank,
                    program
                );
                if let Some((idx, _, _, _, _, _, _, _, _, _, _, _, _, _)) = result {
                    println!("Key: {:>2} -> Sample Index: {}", key, idx);
                }
            }
        } else {
            println!("test.sf2 not found. Skipping percussion test.");
        }
    }
}
