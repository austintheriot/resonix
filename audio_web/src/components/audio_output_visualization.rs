use crate::{
    audio::{
        audio_output_action::AudioOutputAction, play_status::PlayStatus,
        play_status_action::PlayStatusAction,
    },
    state::{
        app_context::{AppContext, AppContextError},
        app_selector::AppSelector,
    },
    utils::animation,
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{function_component, html, prelude::*};

/// Actual canvas height in pixels
pub const AUDIO_OUPTUT_VISUALIZATION_HEIGHT: u32 = 100;

/// Actual maximum canvas width in pixels
pub const AUDIO_OUPTUT_VISUALIZATION_WIDTH: u32 = 100;

#[function_component(AudioOutputVisualization)]
pub fn audio_output_visualization() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let controls_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let animation_frame_handle_ref = use_mut_ref(|| None);
    let buffer_handle = app_context.state_handle.buffer_handle.clone();
    let canvas_ref = use_node_ref();
    let hidden_class = (buffer_handle.get_data().is_empty()
        || !app_context.state_handle.audio_initialized
        || app_context.state_handle.play_status_handle.get() == PlayStatus::Pause)
        .then(|| "hidden");

    use_effect_with_deps(
        {
            let state_handle = app_context.state_handle.clone();
            let canvas_ref = canvas_ref.clone();
            let animation_frame_handle_ref = animation_frame_handle_ref.clone();
            move |_| {
                let clean_up_fn = || {};

                if state_handle.get_are_audio_controls_disabled() {
                    if let Some(animation_frame_handle) = *animation_frame_handle_ref.borrow() {
                        web_sys::window()
                            .unwrap()
                            .cancel_animation_frame(animation_frame_handle)
                            .unwrap();
                    }

                    return clean_up_fn;
                }

                let canvas: HtmlCanvasElement = canvas_ref.cast().unwrap();
                let ctx: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .expect("2D Canvas should be supported")
                    .unwrap()
                    .dyn_into()
                    .unwrap();

                // electric blue
                ctx.set_fill_style(&JsValue::from_str("rgb(31, 159, 209)"));

                // RENDER LOOP
                let f = Rc::new(RefCell::new(None));
                let g = f.clone();
                {
                    let f = f.clone();
                    let state_handle = state_handle.clone();
                    let animation_frame_handle_ref = animation_frame_handle_ref.clone();
                    *g.borrow_mut() =
                        Some(Closure::wrap(Box::new(move || {
                            if state_handle.get_are_audio_controls_disabled() {
                                if let Some(animation_frame_handle) =
                                    *animation_frame_handle_ref.borrow()
                                {
                                    web_sys::window()
                                        .unwrap()
                                        .cancel_animation_frame(animation_frame_handle)
                                        .unwrap();
                                }
                                return;
                            }

                            let moving_average =
                                state_handle.audio_output_handle.get_simple_moving_average();

                            let heights: Vec<f64> = moving_average
                                .clone()
                                .into_iter()
                                .map(|average| {
                                    AUDIO_OUPTUT_VISUALIZATION_HEIGHT as f64 * average as f64
                                })
                                .collect();

                            let ys: Vec<f64> = heights
                                .clone()
                                .into_iter()
                                .map(|height| AUDIO_OUPTUT_VISUALIZATION_HEIGHT as f64 - height)
                                .collect();

                            let width = AUDIO_OUPTUT_VISUALIZATION_WIDTH as f64
                                / moving_average.len() as f64;

                            ctx.clear_rect(
                                0.0,
                                0.0,
                                AUDIO_OUPTUT_VISUALIZATION_WIDTH as f64,
                                AUDIO_OUPTUT_VISUALIZATION_HEIGHT as f64,
                            );

                            heights.iter().enumerate().zip(ys.iter()).for_each(
                                |((i, height), y)| {
                                    let x = i as f64 * width;
                                    ctx.fill_rect(x, *y, width, *height);
                                },
                            );

                            let animation_frame_handle =
                                animation::request_animation_frame((*f).borrow().as_ref().unwrap());
                            *animation_frame_handle_ref.borrow_mut() = Some(animation_frame_handle);
                        })
                            as Box<dyn FnMut()>));
                }

                let animation_frame_handle =
                    animation::request_animation_frame((*g).borrow().as_ref().unwrap());
                *animation_frame_handle_ref.borrow_mut() = Some(animation_frame_handle);

                clean_up_fn
            }
        },
        controls_disabled,
    );

    html! {
        <canvas
            class={classes!("audio-output-visualization", hidden_class)}
            ref={canvas_ref}
            height={AUDIO_OUPTUT_VISUALIZATION_HEIGHT.to_string()}
            width={AUDIO_OUPTUT_VISUALIZATION_WIDTH.to_string()}
        />
    }
}
