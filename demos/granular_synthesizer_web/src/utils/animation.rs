use wasm_bindgen::{prelude::Closure, JsCast};

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK")
}
