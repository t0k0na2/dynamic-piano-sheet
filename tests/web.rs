use wasm_bindgen_test::*;
use wasm_bindgen::JsCast;
use web_sys::{OfflineAudioContext, AudioContext, AudioNode};
use dynamic_piano_sheet::synth::{SoundSource, SynthType};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn test_sound_source_rendering() {
    // 1. Setup OfflineAudioContext (Mono, length 44100 frames = 1 sec, 44100 Hz)
    // OfflineAudioContext inheritance: OfflineAudioContext -> BaseAudioContext -> EventTarget
    let context = OfflineAudioContext::new_with_number_of_channels_and_length_and_sample_rate(1, 44100, 44100.0).unwrap();
    
    // 2. Create the sound source
    // A4 (69), Velocity 100
    let destination = context.destination();
    let _source = SoundSource::new(&context, &destination, 69, 100, 0.0, 1.0, SynthType::FM).unwrap();

    // 3. Start Rendering
    let promise = context.start_rendering().unwrap();
    let result = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    
    // 4. Get AudioBuffer
    let audio_buffer: web_sys::AudioBuffer = result.dyn_into().unwrap();
    
    // 5. Inspect Data
    let channel_data = audio_buffer.get_channel_data(0).unwrap();
    
    // Check first few samples for noise (click)
    // Ideally, sample 0 should be close to 0 or slowly ramping up if attack is handled well.
    let sample0 = channel_data[0];
    let sample1 = channel_data[1];
    let sample10 = channel_data[10];
    
    web_sys::console::log_1(&format!("Sample 0: {}", sample0).into());
    web_sys::console::log_1(&format!("Sample 1: {}", sample1).into());
    web_sys::console::log_1(&format!("Sample 10: {}", sample10).into());

    // Basic assertion: Valid float and not instant explosion
    assert!(!sample0.is_nan());
    assert!(sample0.abs() < 1.0);
    
    // Assert that we have silence at the very beginning if we expected it,
    // or checks that it is non-zero if we expect sound.
    // Since now_time=0.0 and oscillators start at 0.0, sample 0 usually is 0.0.
    // If it is non-zero, it might be a phase discontinuity or click.
    // (Though with Sine wave starting at 0, sin(0)=0).
    // Let's assert it is small.
    assert!(sample0.abs() < 0.1, "Sample 0 is too loud: {}", sample0);
}
