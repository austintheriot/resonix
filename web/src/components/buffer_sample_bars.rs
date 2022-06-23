use crate::state::app_context::{AppContext, AppContextError};
use std::{ops::Mul, sync::Arc};
use yew::{function_component, html, prelude::*};

/// This represents the number of elements to use when showing a buffer sample representation.
///
/// This many divs is slow in development but shows no noticable when running in production mode.
///
/// Other render methods to consider:
/// - Use .svg path <-- this would probably actually be pretty straightforward?
///     - Just use straight vertical lines with a certain thickness?
/// - Use canvas to render
/// - Use css gradient
const NUM_AUDIO_DATA_POINTS: usize = 100;

/// Analyzes a buffer of raw audio sample data into ```NUM_AUDIO_DATA_POINTS``` number of samples,
/// where each sample represents the peak from a chunk of the original audio.
pub fn get_buffer_maxes(buffer: &Arc<Vec<f32>>) -> Vec<String> {
    if buffer.is_empty() {
        return Vec::new();
    }

    // buffer has audio data: get averages from the buffer
    let iteration_group_size = buffer.len() / NUM_AUDIO_DATA_POINTS;
    let maxes: Vec<String> = buffer
        .chunks(iteration_group_size)
        .map(|samples| {
            let max = samples
                .iter()
                .map(|sample| f32::abs(*sample))
                .reduce(f32::max)
                .unwrap()
                .mul(100.0);
            let formatted_max = format!("{:.1}", max);
            formatted_max
        })
        .collect();

    maxes
}

/// A renders a list of divs, the height of which match the
/// amplitude of samples in the context's current buffer.
#[function_component(BufferSampleBars)]
pub fn buffer_sample_bars() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let buffer_maxes = &app_context.state_handle.buffer_maxes;

    if !app_context.state_handle.audio_initialized {
        return html! {};
    }

    if app_context.state_handle.buffer_handle.get_data().is_empty()
        || app_context.state_handle.audio_loading
    {
        return html! {
            <>
                {buffer_maxes.iter().map(|_| {
                    html!{
                        <div class="buffer-sample-bar empty" />
                    }
                }).collect::<Html>()}
            </>
        };
    }

    // buffer has audio data: display it
    html! {
        <>
            {buffer_maxes.iter().map(|percent_string| {
                html!{
                    <div
                        class="buffer-sample-bar"
                        style={format!("transform: scaleY({}%);", percent_string)}
                    />
                }
            }).collect::<Html>()}
        </>
    }
}
