use leptos::{component, Scope, IntoView, view};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <p>"Hello, world!"</p>
    }
}