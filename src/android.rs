//! Android Native Activity Entry Point
//!
//! Provides the pure Rust entrypoint for `android-activity` / `NativeActivity`.

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use std::path::PathBuf;

    let internal_data_path = app
        .internal_data_path()
        .unwrap_or_else(|| PathBuf::from("."));
    let storage_path = internal_data_path.join("todos.json");

    let mut native_options = eframe::NativeOptions::default();
    native_options.android_app = Some(app);

    let _ = eframe::run_native(
        "Todo",
        native_options,
        Box::new(move |_cc| Ok(Box::new(crate::gui::TodoGuiApp::new(Some(storage_path))))),
    );
}
