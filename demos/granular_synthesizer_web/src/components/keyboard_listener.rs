use crate::state::{
    app_action::AppAction,
    app_context::{AppContext, AppContextError},
};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct KeyboardListenerProps {
    pub children: Children,
}

#[function_component(KeyboardListener)]
pub fn keyboard_listener(props: &KeyboardListenerProps) -> Html {
    let app_context = use_context::<AppContext>().expect(AppContextError::NOT_FOUND);
    let callback_handle = use_mut_ref(|| None);
    use_effect_with_deps(
        move |_| {
            let handle_keydown = Closure::wrap(Box::new(move |e: KeyboardEvent| {
                if app_context.state_handle.is_keyboard_user {
                    return;
                }
                if let "Tab" = e.key().as_str() {
                    app_context
                        .state_handle
                        .dispatch(AppAction::SetIsKeyboardUser);
                }
            }) as Box<dyn FnMut(KeyboardEvent)>);

            window()
                .unwrap()
                .set_onkeydown(Some(handle_keydown.as_ref().unchecked_ref()));

            // keep callback valid for component lifecycle
            callback_handle.borrow_mut().replace(handle_keydown);

            // invalidate callback and remove on unmount
            move || {
                if let Some(callback) = &*callback_handle.borrow() {
                    window()
                        .unwrap()
                        .remove_event_listener_with_callback(
                            "onkeydown",
                            callback.as_ref().unchecked_ref(),
                        )
                        .expect("Should be able to remove keyboard listener from window");
                    callback_handle.borrow_mut().take();
                }
            }
        },
        (),
    );

    html! {
     <>
         {for props.children.iter()}
     </>
    }
}
