use crate::state::app_context::{AppContext, AppContextError};
use std::ops::Mul;
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

/// A renders a list of divs, the height of which match the
/// amplitude of samples in the context's current buffer.
#[function_component(BufferSampleBars)]
pub fn buffer_sample_bars() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    
    // empty buffer
    if app_context.state_handle.buffer.data.is_empty() {
        return html! {
            <>
                {(0..NUM_AUDIO_DATA_POINTS).into_iter().map(|_| {
                    html!{
                        <div class="buffer-visualizer__audio-bar buffer-empty" />
                    }
                }).collect::<Html>()}
            </>
        };
    }

    // buffer has audio data: display it
    let iteration_group_size = app_context.state_handle.buffer.data.len() / NUM_AUDIO_DATA_POINTS;
    let sample_averages: Vec<String> = app_context
        .state_handle
        .buffer
        .data
        .chunks(iteration_group_size)
        .map(|samples| {
            let sum = samples.iter().map(|sample| sample.abs()).sum::<f32>();
            let average_percent = (sum / samples.len() as f32).mul(100.0).min(100.0).max(1.0);
            let formatted_percent = format!("{:.1}", average_percent);
            formatted_percent
        })
        .collect();

    html! {
        <>
            {sample_averages.iter().map(|percent_string| {
                html!{
                    <div 
                        class="buffer-visualizer__audio-bar" 
                        style={format!("transform: scaleY({}%);", percent_string)} 
                    />
                }
            }).collect::<Html>()}
        </>
    }
}
