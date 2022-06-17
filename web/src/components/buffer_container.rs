use crate::components::buffer_selection::BufferSelectionVisualizer;
use crate::components::buffer_sample_bars::BufferSampleBars;
use crate::state::app_action::AppAction;
use crate::state::app_context::{AppContext, AppContextError};
use log::info;
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
    let mouse_start_point: UseStateHandle<Option<f32>> = use_state(|| None);
    let mouse_end_point: UseStateHandle<Option<f32>> = use_state(|| None);
    let mouse_down = use_state(|| false);

    info!("start {:?}, end = {:?}, down = {:?}", *mouse_start_point, *mouse_end_point, *mouse_down);
    
    let handle_mouse_down = {
        let div_ref = div_ref.clone();
        let mouse_down = mouse_down.clone();
        let mouse_start_point = mouse_start_point.clone();
        let mouse_end_point = mouse_end_point.clone();
        Callback::from(move |e: MouseEvent| {
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let start_point = (e.offset_x() as f32) / (div.client_width() as f32);
            mouse_down.set(true);
            mouse_start_point.set(Some(start_point));
            mouse_end_point.set(Some(start_point));
        })
    };

    let handle_mouse_up = {
        let div_ref = div_ref.clone();
        let mouse_down = mouse_down.clone();
        let mouse_end_point = mouse_end_point.clone();
        Callback::from(move |e: MouseEvent| {
            let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
            let end_point = (e.offset_x() as f32) / (div.client_width() as f32);
            mouse_down.set(false);
            mouse_end_point.set(Some(end_point));
        })
    };

    let handle_mouse_leave = {
        let mouse_down = mouse_down.clone();
        Callback::from(move |_: MouseEvent| {
            mouse_down.set(false);
        })
    };

    let handle_mouse_move = {
        let div_ref = div_ref.clone();
        let mouse_end_point = mouse_end_point.clone();
        Callback::from(move |e: MouseEvent| {
            if *mouse_down {
                let div = div_ref.get().unwrap().dyn_into::<HtmlDivElement>().unwrap();
                let end_point = (e.offset_x() as f32) / (div.client_width() as f32);
                mouse_end_point.set(Some(end_point));
            }
        })
    };

    let div_ref_prop = div_ref.clone();
    let buffer_selection_visualizer = if let (Some(mouse_start_point), Some(mouse_end_point)) = (*mouse_start_point, *mouse_end_point) {
        let (slider_start_point, slider_end_point) = if mouse_start_point > mouse_end_point {
            (mouse_end_point, mouse_start_point)
        } else {
            (mouse_start_point, mouse_end_point)
        };

        let slider_start_point = slider_start_point.max(0.0).min(1.0);
        let slider_end_point = slider_end_point.max(0.0).min(1.0);

        html!{
            <BufferSelectionVisualizer div_ref={div_ref_prop} start={slider_start_point} end={slider_end_point} />
        }
    } else {
        html!{}
    };


    html! {
        <div
            class="buffer-visualizer"
            onmousedown={handle_mouse_down}
            onmouseup={handle_mouse_up}
            onmouseleave={handle_mouse_leave}
            onmousemove={handle_mouse_move}
            ref={div_ref}
        >
            {buffer_selection_visualizer}
            <BufferSampleBars />
        </div>
    }
}
