use crate::components::buffer_selection::BufferSelectionVisualizer;
use crate::state::app_action::AppAction;
use crate::state::app_context::{AppContext, AppContextError};
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::{function_component, html, prelude::*};

/// A wrapper around the audio buffer visualization
///
/// This component is responsible for handling all the mouse / touch interactions
#[function_component(BufferVisualizer)]
pub fn buffer_visualizer() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let div_ref = use_node_ref();

    let handle_mouse_down = {
        let state_handle = app_context.state_handle.clone();
        let div_ref = div_ref.clone();
        Callback::from(move |e: MouseEvent| {
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let buffer_selection_start = (e.offset_x() as f32) / (div.client_width() as f32);
            state_handle.dispatch(AppAction::SetBufferSelectionStart(buffer_selection_start));
            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(true));
        })
    };

    let handle_mouse_up = {
        let state_handle = app_context.state_handle.clone();
        let div_ref = div_ref.clone();
        Callback::from(move |e: MouseEvent| {
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let buffer_selection_end = (e.offset_x() as f32) / (div.client_width() as f32);
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(buffer_selection_end));
            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
        })
    };

    let handle_mouse_leave = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
        })
    };

    let handle_mouse_move = {
        let state_handle = app_context.state_handle.clone();
        let div_ref = div_ref.clone();
        Callback::from(move |e: MouseEvent| {
            let is_mouse_down = state_handle.buffer_selection.as_ref().map(|buffer_selection| {
                buffer_selection.mouse_down
            }).unwrap_or(false);

            if is_mouse_down {
                let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
                let buffer_selection_end = (e.offset_x() as f32) / (div.client_width() as f32);
                state_handle.dispatch(AppAction::SetBufferSelectionEnd(buffer_selection_end));
            }
        })
    };

    let div_ref_prop = div_ref.clone();

    html! {
        <div
            class="buffer-visualizer"
            onmousedown={handle_mouse_down}
            onmouseup={handle_mouse_up}
            onmouseleave={handle_mouse_leave}
            onmousemove={handle_mouse_move}
            ref={div_ref}
        >
            <BufferSelectionVisualizer div_ref={div_ref_prop} />
            // <BufferSampleBars />
        </div>
    }
}
