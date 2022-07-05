use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{AudioBuffer, AudioContext};

#[derive(Error, Debug)]
pub enum DecodeBytesError {
    #[error("failed to decode audio data")]
    DecodeFailure,
    #[error("failed to convert promise to future")]
    PromiseToFutureFailure,
    #[error("failed to convert JavaScript type to expected Rust type")]
    TypeConversionFailure,
}

/// Decodes raw bytes into a JavaScript AudioBuffer, using the browser's built-in `decode_audio_data` functionality.
pub async fn decode_bytes(
    audio_context: &AudioContext,
    bytes: &[u8],
) -> Result<AudioBuffer, DecodeBytesError> {
    // This action is "unsafe" because it's creating a JavaScript view into wasm linear memory.
    // This is low risk as long as no allocations are made between this call and `decode_audio_data`.
    let mp3_u_int8_array = unsafe { js_sys::Uint8Array::view(bytes) };

    // this data must be copied, because decodeAudioData() claims the ArrayBuffer it receives
    let mp3_u_int8_array = mp3_u_int8_array.slice(0, mp3_u_int8_array.length());

    let decoded_audio_promise = audio_context
        .decode_audio_data(&mp3_u_int8_array.buffer())
        .map_err(|_| DecodeBytesError::DecodeFailure)?;

    let audio_buffer: web_sys::AudioBuffer =
        wasm_bindgen_futures::JsFuture::from(decoded_audio_promise)
            .await
            .map_err(|_| DecodeBytesError::PromiseToFutureFailure)?
            .dyn_into()
            .map_err(|_| DecodeBytesError::TypeConversionFailure)?;

    Ok(audio_buffer)
}
