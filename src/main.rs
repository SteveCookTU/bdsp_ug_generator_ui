#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bdsp_ug_generator_ui::BDSPUgGeneratorUI;
use eframe::egui::vec2;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = false;
    native_options.resizable = false;
    native_options.initial_window_size = Some(vec2(500.0, 500.0));
    eframe::run_native(
        "BDSP Underground Generator",
        native_options,
        Box::new(|cc| Box::new(BDSPUgGeneratorUI::new(cc))),
    );
}
