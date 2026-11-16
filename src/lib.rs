pub mod gui;
pub mod models;

#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod server;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod tui;
