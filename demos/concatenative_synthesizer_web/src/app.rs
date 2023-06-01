use leptos::{component, view, IntoView, Scope};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <p>"Hello, world!"</p>
    }
}
