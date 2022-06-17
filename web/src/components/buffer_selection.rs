use crate::state::app_context::{AppContext, AppContextError};
use std::ops::Sub;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::{function_component, html, prelude::*};

#[derive(Properties, PartialEq)]
pub struct BufferSelectionProps {
    pub div_ref: NodeRef,
    pub start: f32,
    pub end: f32,
}

#[function_component(BufferSelectionVisualizer)]
pub fn buffer_selection_visualizer(props: &BufferSelectionProps) -> Html {
    let div_width = if let Some(div) = props.div_ref.get() {
        div.dyn_into::<HtmlDivElement>().unwrap().client_width() as f32
    } else {
        0.0
    };
    let translate_x_in_px = format!("{:.2}", props.start * div_width);
    let scale_x_in_percent = format!("{:.3}", props.end.sub(props.start));
    let selection_style = format!(
        "transform: translateX({}px) scale({}, 1.0);",
        translate_x_in_px, scale_x_in_percent
    );

    html! {
        <div class="buffer-visualizer-selection" style={selection_style} />
    }
}
