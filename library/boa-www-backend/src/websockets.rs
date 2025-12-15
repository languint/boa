use wasm_bindgen::{
    JsCast, JsValue,
    prelude::{Closure, wasm_bindgen},
};
use web_sys::{ErrorEvent, MessageEvent, WebSocket, js_sys};

use crate::log;

#[wasm_bindgen]
pub fn connect_ws(url: String) -> Result<(), JsValue> {
    let ws = WebSocket::new(&url)?;
    log(&format!("connecting to {url}"));

    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let cloned_ws = ws.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            log(&format!("message event, received arraybuffer: {:?}", abuf));

            let array = js_sys::Uint8Array::new(&abuf);
            let len = array.byte_length() as usize;

            log(&format!(
                "Arraybuffer received {}bytes: {:?}",
                len,
                array.to_vec()
            ));

            cloned_ws.set_binary_type(web_sys::BinaryType::Blob);
            match cloned_ws.send_with_u8_array(&[5, 6, 7, 8]) {
                Ok(_) => log(&format!("binary message successfully sent")),
                Err(err) => log(&format!("error sending message: {:?}", err)),
            }
        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            log(&format!("message event, received blob: {:?}", blob));

            let fr = web_sys::FileReader::new().unwrap();
            let fr_c = fr.clone();

            let onloadend_cb = Closure::<dyn FnMut(_)>::new(move |_e: web_sys::ProgressEvent| {
                let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
                let len = array.byte_length() as usize;
                log(&format!("Blob received {}bytes: {:?}", len, array.to_vec()));
            });
            fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
            fr.read_as_array_buffer(&blob).expect("blob not readable");
            onloadend_cb.forget();
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            log(&format!("message event, received Text: {:?}", txt));
        } else {
            log(&format!("message event, received Unknown: {:?}", e.data()));
        }
    });

    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        log(&format!("error event: {:?}", e));
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let cloned_ws = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        log("socket opened");
        match cloned_ws.send_with_str("ping") {
            Ok(_) => log("message successfully sent"),
            Err(err) => log(&format!("error sending message: {:?}", err)),
        }

        match cloned_ws.send_with_u8_array(&[0, 1, 2, 3]) {
            Ok(_) => log("binary message successfully sent"),
            Err(err) => log(&format!("error sending message: {:?}", err)),
        }
    });
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    Ok(())
}
