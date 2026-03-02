mod app;
mod logic;
mod models;

use app::OptimizationApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Методы одномерной оптимизации",
        options,
        Box::new(|_cc| Ok(Box::new(OptimizationApp::default()))),
    )
}
