mod app;
mod utils;
mod tabs;
mod settings;

use app::OptimizationApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Методы оптимизации - Лабораторные работы",
        options,
        Box::new(|_cc| Ok(Box::new(OptimizationApp::default()))),
    )
}
