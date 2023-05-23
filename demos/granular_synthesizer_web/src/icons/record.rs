use yew::{function_component, html};

#[function_component(IconRecord)]
pub fn icon_record() -> Html {
    html! {
        <svg class="feather feather-circle" width="24" height="24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" version="1.1" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <circle cx="12" cy="12" r="10"/>
            <circle cx="12" cy="12" r="4.1032" fill="currentColor" stroke-width="2"/>
        </svg>
    }
}
