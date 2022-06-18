use crate::state::app_context::{AppContext, AppContextError};
use std::ops::Sub;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::{function_component, html, prelude::*};

#[derive(Properties, PartialEq)]
pub struct BufferSelectionProps {
    pub div_ref: NodeRef,
}

#[function_component(BufferSelectionVisualizer)]
pub fn buffer_selection_visualizer(props: &BufferSelectionProps) -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let (start, end) = &app_context
        .state_handle
        .buffer_handle
        .buffer_selection
        .lock()
        .unwrap()
        .get_buffer_start_and_end();

    let div_width = if let Some(div) = props.div_ref.get() {
        div.dyn_into::<HtmlDivElement>().unwrap().client_width() as f32
    } else {
        0.0
    };
    let translate_x_in_px = format!("{:.2}", start * div_width);
    let scale_x_in_percent = format!("{:.3}", end.sub(start));
    let selection_style = format!(
        "transform: translateX({}px) scale({}, 1.0);",
        translate_x_in_px, scale_x_in_percent
    );

    html! {
        <div class="buffer-selection-visualizer" style={selection_style} />
    }
}
