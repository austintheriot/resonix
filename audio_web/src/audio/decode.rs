use wasm_bindgen::JsCast;
use web_sys::AudioContext;

/// Decodes raw bytes into a JavaScript AudioBuffer, using the browser's built-in `decode_audio_data` functionality.
pub async fn decode_bytes(audio_context: &AudioContext, bytes: &[u8]) -> web_sys::AudioBuffer {
    // This action is "unsafe" because it's creating a JavaScript view into wasm linear memory.
    // This is low risk as long as no allocations are made between this call and `decode_audio_data`.
    let mp3_u_int8_array = unsafe { js_sys::Uint8Array::view(bytes) };

    // this data must be copied, because decodeAudioData() claims the ArrayBuffer it receives
    let mp3_u_int8_array = mp3_u_int8_array.slice(0, mp3_u_int8_array.length());

    let decoded_audio_promise = audio_context
        .decode_audio_data(&mp3_u_int8_array.buffer())
        .expect("Should succeed at decoding audio data");

    let audio_buffer: web_sys::AudioBuffer =
        wasm_bindgen_futures::JsFuture::from(decoded_audio_promise)
            .await
            .expect("Should convert decode_audio_data Promise into Future")
            .dyn_into()
            .expect("decode_audio_data should return a buffer of data on success");

    audio_buffer
}
