pub mod gui;
pub mod models;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
pub mod cli;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
pub mod tui;

pub mod android;
pub mod ios;
