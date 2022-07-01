use wasm_bindgen::JsCast;
use web_sys::{Blob, HtmlAnchorElement, Url};

/// Downloads raw slice of bytes as a file from the browser
pub fn download_bytes(bytes: impl AsRef<[u8]>, file_name: &str) {
    let bytes = bytes.as_ref();
    // make all wasm memory allocations at the beginning of the function
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let a: HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();
    a.style().set_css_text("display: none;");
    a.set_download(file_name);
    body.append_child(&a).unwrap();

    // data must be passed to blob constructor inside of a javascript array
    let blob_parts = js_sys::Array::new_with_length(1);
    
    // it is unsafe to get a raw view into WebAssembly memory, but because this memory gets imemdiately
    // used, downloaded, and then view is discarded, it is safe so long as no new allocations are 
    // made in between acquiring the view and using it
    let u8_view = unsafe { js_sys::Uint8Array::view(bytes) };
    blob_parts.set(0, u8_view.dyn_into().unwrap());

    // create blob from raw view into wasm linear memory
    let blob =
        Blob::new_with_buffer_source_sequence(&blob_parts.as_ref()).unwrap();
    
    // make blob downloadable by creating a global document url for the blob resource
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    a.set_href(&url);
    a.click();

    // release url from window memory when done to prevent memory leak
    // (this does not get released automatically, unlike most of web memory)
    Url::revoke_object_url(&url).unwrap();
}