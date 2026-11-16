#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let document = match window.document() {
            Some(d) => d,
            None => return,
        };

        let canvas = match document.get_element_by_id("the_canvas_id") {
            Some(el) => match el.dyn_into::<web_sys::HtmlCanvasElement>() {
                Ok(c) => c,
                Err(_) => return,
            },
            None => return,
        };

        let _ = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(todo::gui::TodoGuiApp::new(None)))),
            )
            .await;
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Todo Web target is intended to be built with Trunk for wasm32-unknown-unknown.");
}
