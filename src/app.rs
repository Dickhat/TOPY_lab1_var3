use eframe::egui;
use crate::models::OptimizationResult;
use crate::logic::{self, Func};
use std::cell::Cell;

pub struct OptimizationApp {
    a: f64, b: f64, eps: f64, l: f64,
    selected_func: usize,
    selected_method: usize,
    result: Option<OptimizationResult>,
}

impl Default for OptimizationApp {
    fn default() -> Self {
        Self {
            a: -3.0, b: 3.0, eps: 0.01, l: 0.1,
            selected_func: 0, selected_method: 0,
            result: None,
        }
    }
}

impl OptimizationApp {
    fn run_calc(&mut self) {
        let f_raw = |x: f64| match self.selected_func {
            0 => 3.0 * x - x.powi(3),
            1 => (9.0 - x.powi(2)) / (x.powi(2) + 2.0 * x + 3.0),
            _ => 0.0,
        };
        
        let f = Func { f: &f_raw, calls: Cell::new(0) };
        let is_max = self.selected_func == 0;

        self.result = Some(match self.selected_method {
            0 => logic::dichotomy(self.a, self.b, self.eps, self.l, &f, is_max),
            // 1 => logic::golden_ratio(...),
            _ => logic::dichotomy(self.a, self.b, self.eps, self.l, &f, is_max), // заглушка
        });
    }
}

impl eframe::App for OptimizationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Код отрисовки панелей, кнопок и графиков (как в вашем примере)
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Рассчитать").clicked() {
                self.run_calc();
            }
            // ... вывод таблицы и графиков ...
        });
    }
}