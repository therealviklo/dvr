use wasm_bindgen::prelude::*;
use wasm_bindgen::convert::FromWasmAbi;
use web_sys::EventTarget;

// pub fn log_errors_arg<T: FromWasmAbi + 'static>(mut f: impl FnMut(T) -> Result<(), JsValue>) -> impl FnMut(T) {
//     move |t: T| {
//         if let Err(e) = f(t) {
//             web_sys::console::error_1(&e)
//         }
//     }
// }

pub fn log_errors(mut f: impl FnMut() -> Result<(), JsValue>) -> impl FnMut() {
    move || {
        if let Err(e) = f() {
            web_sys::console::error_1(&e)
        }
    }
}

pub fn add_event_listener<T: FromWasmAbi + 'static>(val: &EventTarget, event: &str, f: impl FnMut(T) + 'static) -> Result<Closure<dyn FnMut(T)>, JsValue>
{
    let closure = Closure::<dyn FnMut(_)>::new(f);
    val.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())?;
    Ok(closure)
}

pub fn js_val_err_to_string(e: JsValue) -> String {
    e.as_string()
        .unwrap_or_else(|| "Unknown JS error".to_string())
}