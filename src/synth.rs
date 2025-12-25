use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, AudioNode, BiquadFilterNode, GainNode, OscillatorNode, BiquadFilterType, OscillatorType};

pub struct SoundSource {
    vco: OscillatorNode,
    vcf: BiquadFilterNode,
    vca: GainNode,
    now_time: f64,
    end_time: f64,
}

impl SoundSource {
    pub fn new(context: &AudioContext, destination_target: &AudioNode, key: u8, velocity: u8, start_time: f64, end_time: f64) -> Result<SoundSource, JsValue> {
        let base_gain = Self::velocity_to_ratio(velocity);
        let vca_a = 0.1;
        let vca_d = 0.2;
        let vca_s = 0.5 * base_gain;
        let vca_r = 1.2;
        let end_time = end_time.max(start_time + vca_a + vca_d);

        let freq = Self::midi_key_to_freq(key);
        let vco = context.create_oscillator()?;
        vco.set_type(OscillatorType::Sawtooth);
        vco.frequency().set_value(freq);

        let vcf = context.create_biquad_filter()?;
        vcf.set_type(BiquadFilterType::Lowpass);
        vcf.frequency().set_value((freq * 4.0).min(10000.0));
        vcf.frequency().linear_ramp_to_value_at_time(freq * 0.5, end_time)?;
        
        let vca = context.create_gain()?;
        vca.gain().set_value_at_time(0.0, start_time)?;
        vca.gain().linear_ramp_to_value_at_time(base_gain, start_time + vca_a)?;
        vca.gain().linear_ramp_to_value_at_time(vca_s, start_time + vca_a + vca_d)?;
        vca.gain().linear_ramp_to_value_at_time(vca_s, end_time)?;
        vca.gain().linear_ramp_to_value_at_time(0.0001, end_time + vca_r)?;
        
        vco.connect_with_audio_node(&vcf)?;
        vcf.connect_with_audio_node(&vca)?;
        vca.connect_with_audio_node(destination_target)?;
        vco.start_with_when(start_time)?;

        Ok(SoundSource {
            vco,
            vcf,
            vca,
            now_time: start_time,
            end_time: end_time,
        })
    }

    fn velocity_to_ratio(velocity: u8) -> f32{
        velocity as f32 / 127.0
    }

    pub fn tick(&mut self, delta_sec: f64) {
        self.now_time += delta_sec;
    }

    pub fn finished(&self) -> bool {
        self.now_time >= self.end_time && self.vca.gain().value() <= 0.001
    }

    fn midi_key_to_freq(key: u8) -> f32 {
        27.5 * 2f32.powf((key as f32 - 21.0) / 12.0)
    }
}

impl Drop for SoundSource {
    fn drop(&mut self) {
        self.vco.disconnect().unwrap();
        self.vcf.disconnect().unwrap();
        self.vca.disconnect().unwrap();
    }
}