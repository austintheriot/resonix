use crate::components::buffer_sample_bars::BufferSampleBars;
use crate::components::buffer_selection_visualizer::BufferSelectionVisualizer;
use crate::state::app_action::AppAction;
use crate::state::app_context::{AppContext, AppContextError};
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::{function_component, html, prelude::*};

/// A wrapper around the audio buffer visualization
///
/// This component is responsible for handling all the mouse / touch interactions
#[function_component(BufferContainer)]
pub fn buffer_container() -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let div_ref = use_node_ref();

    let handle_mouse_down = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: MouseEvent| {
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
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let end_point = (e.offset_x() as f32) / (div.client_width() as f32);

            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
            state_handle.dispatch(AppAction::SetBufferSelectionEnd(end_point));
        })
    };

    let handle_mouse_leave = {
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |_: MouseEvent| {
            state_handle.dispatch(AppAction::SetBufferSelectionMouseDown(false));
        })
    };

    let handle_mouse_move = {
        let div_ref = div_ref.clone();
        let state_handle = app_context.state_handle.clone();
        Callback::from(move |e: MouseEvent| {
            let mouse_down = state_handle.buffer_selection_handle.get_mouse_down();

            if mouse_down {
                let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
                let end_point = (e.offset_x() as f32) / (div.client_width() as f32);

                state_handle.dispatch(AppAction::SetBufferSelectionEnd(end_point));
            }
        })
    };

    let div_ref_prop = div_ref.clone();

    html! {
        <div
            class="buffer-container"
            onmousedown={handle_mouse_down}
            onmouseup={handle_mouse_up}
            onmouseleave={handle_mouse_leave}
            onmousemove={handle_mouse_move}
            ref={div_ref}
        >
            <BufferSelectionVisualizer div_ref={div_ref_prop} />
            <BufferSampleBars />
        </div>
    }
}
