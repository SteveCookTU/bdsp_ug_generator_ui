#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bdsp_ug_generator_ui::BDSPUgGeneratorUI;
use eframe::egui::vec2;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    tracing_subscriber::fmt::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = false;
    native_options.resizable = false;
    native_options.min_window_size = Some(vec2(500.0, 700.0));
    eframe::run_native(
        "BDSP Underground Generator",
        native_options,
        Box::new(|cc| Box::new(BDSPUgGeneratorUI::new(cc))),
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "bdsp_ug_generator", // hardcode it
        web_options,
        Box::new(|cc| Box::new(BDSPUgGeneratorUI::new(cc))),
    )
    .expect("failed to start eframe");
}
