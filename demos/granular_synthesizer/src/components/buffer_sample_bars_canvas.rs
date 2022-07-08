use crate::state::app_context::{AppContext, AppContextError};

use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{function_component, html, prelude::*};

/// This represents the number of elements to use when showing a buffer sample representation.
const BUFFER_SAMPLE_BARS_CANVAS_NUM_DATA_POINTS: usize = 100;

/// Analyzes a buffer of raw audio sample data into ```NUM_AUDIO_DATA_POINTS``` number of samples,
/// where each sample represents the peak from a chunk of the original audio.
///
/// Formatted as 0.0 -> 1.0
pub fn get_buffer_maxes_for_canvas(buffer: &Arc<Vec<f32>>) -> Vec<f32> {
    if buffer.is_empty() {
        return Vec::new();
    }

    // buffer has audio data: get averages from the buffer
    let iteration_group_size = buffer.len() / BUFFER_SAMPLE_BARS_CANVAS_NUM_DATA_POINTS;
    let maxes: Vec<f32> = buffer
        .chunks(iteration_group_size)
        .map(|samples| {
            samples
                .iter()
                .map(|sample| f32::abs(*sample))
                .reduce(f32::max)
                .unwrap()
        })
        .collect();

    maxes
}

/// Actual canvas height in pixels
pub const BUFFER_SAMPLE_BARS_CANVAS_HEIGHT: u32 = 80;

/// Actual maximum canvas width in pixels
pub const BUFFER_SAMPLE_BARS_CANVAS_WIDTH: u32 = 718;

/// This represents the size the sample visualization bar can take as a percentage of its max size
pub const BUFFER_SAMPLE_BARS_CANVAS_BAR_MAX_WIDTH_PERCENTAGE: f32 = 0.8;

/// This is the static width of a sample bar in canvas pixels
/// (i.e. relative to the canvas rather than true pixels)
pub const BUFFER_SAMPLE_BARS_CANVAS_BAR_WIDTH: f32 = (BUFFER_SAMPLE_BARS_CANVAS_WIDTH as f32
    / BUFFER_SAMPLE_BARS_CANVAS_NUM_DATA_POINTS as f32)
    * BUFFER_SAMPLE_BARS_CANVAS_BAR_MAX_WIDTH_PERCENTAGE;

/// A renders a graphical representation of the current buffer's max amplitudes
#[function_component(BufferSampleBarsCanvas)]
pub fn buffer_sample_bars_canvas() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let buffer_handle = app_context.state_handle.buffer_handle.clone();
    let canvas_ref = use_node_ref();
    let hidden_class = (buffer_handle.get_data().is_empty()
        || !app_context.state_handle.audio_initialized)
        .then(|| "hidden");

    use_effect_with_deps(
        {
            let canvas_ref = canvas_ref.clone();
            move |_| {
                let canvas: HtmlCanvasElement = canvas_ref.cast().unwrap();
                let ctx: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .expect("2D Canvas should be supported")
                    .unwrap()
                    .dyn_into()
                    .unwrap();

                let buffer_maxes = &app_context.state_handle.buffer_maxes_for_canvas;
                ctx.clear_rect(
                    0.0,
                    0.0,
                    BUFFER_SAMPLE_BARS_CANVAS_WIDTH as f64,
                    BUFFER_SAMPLE_BARS_CANVAS_HEIGHT as f64,
                );
                for (i, amplitude) in buffer_maxes.iter().enumerate() {
                    let height = amplitude * BUFFER_SAMPLE_BARS_CANVAS_HEIGHT as f32;
                    let x_percent = i as f32 / buffer_maxes.len() as f32;
                    let x = x_percent * BUFFER_SAMPLE_BARS_CANVAS_WIDTH as f32;
                    let y = (BUFFER_SAMPLE_BARS_CANVAS_HEIGHT as f32 - height) / 2.0;

                    ctx.set_fill_style(&JsValue::from_str("black"));
                    ctx.fill_rect(
                        x as f64,
                        y as f64,
                        BUFFER_SAMPLE_BARS_CANVAS_BAR_WIDTH as f64,
                        height as f64,
                    );
                }
                || {}
            }
        },
        buffer_handle,
    );

    html! {
        <canvas
            class={classes!("buffer-sample-bars-canvas", hidden_class)}
            ref={canvas_ref}
            height={BUFFER_SAMPLE_BARS_CANVAS_HEIGHT.to_string()}
            width={BUFFER_SAMPLE_BARS_CANVAS_WIDTH.to_string()}
        />
    }
}
