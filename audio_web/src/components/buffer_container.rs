use crate::audio::buffer_selection_action::BufferSelectionAction;
use crate::audio::play_status::PlayStatus;
use crate::audio::play_status_action::PlayStatusAction;
use crate::components::buffer_sample_bars::BufferSampleBars;
use crate::components::buffer_selection_visualizer::BufferSelectionVisualizer;
use crate::state::app_action::AppAction;
use crate::state::app_context::{AppContext, AppContextError};
use crate::state::app_selector::AppSelector;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::{function_component, html, prelude::*};

/// Calculate the current touch location within the div, as a percentage (0.0 -> 1.0)
pub fn get_touch_percent_x(div_ref: &NodeRef, touch_client_x: i32) -> f32 {
    let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
    let div_rect = div.get_bounding_client_rect();
    let div_x = div_rect.x() as f32;
    // the x coordinate of the touch, relative to the left edge of the div
    let touch_el_x = (touch_client_x as f32) - div_x;
    let div_width = div_rect.width() as f32;

    touch_el_x / div_width
}

/// A wrapper around the audio buffer visualization
///
/// This component is responsible for handling all the mouse / touch interactions
#[function_component(BufferContainer)]
pub fn buffer_container() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let buffer_selector_disabled = app_context.state_handle.get_are_audio_controls_disabled();
    let div_ref = use_node_ref();

    let handle_mouse_down = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            if buffer_selector_disabled {
                return;
            }
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let start_point = (e.offset_x() as f32) / (div.client_width() as f32);

            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(true));
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(start_point));
            state_handle.dispatch(AppAction::SetBufferSelectionStart(start_point));
        })
    };

    let handle_mouse_up = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: MouseEvent| {
            if buffer_selector_disabled {
                return;
            }
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let end_point = (e.offset_x() as f32) / (div.client_width() as f32);

            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(end_point));
        })
    };

    let handle_mouse_leave = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            if buffer_selector_disabled {
                return;
            }
            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
        })
    };

    let handle_mouse_move = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();

            if buffer_selector_disabled {
                return;
            }
            let mouse_down = state_handle.buffer_selection_handle.get_mouse_down();

            if mouse_down {
                let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
                let end_point = (e.offset_x() as f32) / (div.client_width() as f32);

                state_handle.dispatch(AppAction::SetBufferSelectionEnd(end_point));
            }
        })
    };

    let handle_touch_start = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: TouchEvent| {
            if buffer_selector_disabled {
                return;
            }
            let touch = e
                .touches()
                // ignore any multi-touches
                .get(0)
                .expect("There should be at least one touch in the touch list");
            let touch_client_x = touch.client_x();
            let touch_percent_x = get_touch_percent_x(&div_ref, touch_client_x);

            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(true));
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(touch_percent_x));
            state_handle.dispatch(AppAction::SetBufferSelectionStart(touch_percent_x));
        })
    };

    let handle_touch_end = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: TouchEvent| {
            if buffer_selector_disabled {
                return;
            }
            let touch = e
                .changed_touches()
                // ignore any multi-touches
                .get(0)
                .expect("There should be at least one touch in the touch list");
            let touch_client_x = touch.client_x();
            let touch_percent_x = get_touch_percent_x(&div_ref, touch_client_x);

            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(touch_percent_x));
        })
    };

    let handle_touch_move = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: TouchEvent| {
            if buffer_selector_disabled {
                return;
            }
            let touch = e
                .changed_touches()
                // ignore any multi-touches
                .get(0)
                .expect("There should be at least one touch in the touch list");
            let touch_client_x = touch.client_x();
            let touch_percent_x = get_touch_percent_x(&div_ref, touch_client_x);

            state_handle.dispatch(AppAction::SetBufferSelectionEnd(touch_percent_x));
        })
    };

    let handle_key_down = {
        let state_handle = app_context.state_handle;
        Callback::from(move |e: KeyboardEvent| {
            if buffer_selector_disabled {
                return;
            }

            e.prevent_default();
            let shift_key = e.shift_key();
            match e.key().as_str() {
                "ArrowRight" | "Right" if shift_key => {
                    state_handle.dispatch(AppAction::IncrementBufferSelectionEnd);
                }
                "ArrowLeft" | "Left" if shift_key => {
                    state_handle.dispatch(AppAction::DecrementBufferSelectionEnd);
                }
                "ArrowRight" | "Right" => {
                    state_handle.dispatch(AppAction::IncrementBufferSelectionStart);
                }
                "ArrowLeft" | "Left" => {
                    state_handle.dispatch(AppAction::DecrementBufferSelectionStart);
                }
                " " => {
                    let new_play_status = match state_handle.play_status_handle.get() {
                        PlayStatus::Play => PlayStatus::Pause,
                        PlayStatus::Pause => PlayStatus::Play,
                    };
                    state_handle.dispatch(AppAction::SetPlayStatus(new_play_status));
                }
                _ => {}
            }
        })
    };

    let div_ref_prop = div_ref.clone();
    let tab_index = if buffer_selector_disabled { "-1" } else { "0" };

    html! {
        <div
            class="buffer-container"
            onmousedown={handle_mouse_down}
            onmouseup={handle_mouse_up}
            onmouseleave={handle_mouse_leave}
            onmousemove={handle_mouse_move}
            ontouchstart={handle_touch_start}
            ontouchend={handle_touch_end}
            ontouchmove={handle_touch_move}
            onkeydown={handle_key_down}
            tabindex={tab_index}
            ref={div_ref}
            data-disabled={buffer_selector_disabled.to_string()}
        >
            <BufferSampleBars />
            <BufferSelectionVisualizer div_ref={div_ref_prop} />
        </div>
    }
}
