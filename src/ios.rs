//! iOS Native App Entry Point
//!
//! Exposes a pure-Rust C-ABI entrypoint to run the egui/eframe application on iOS.

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn run_ios_app() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Todo",
        native_options,
        Box::new(|_cc| Ok(Box::new(crate::gui::TodoGuiApp::new(None)))),
    );
}
