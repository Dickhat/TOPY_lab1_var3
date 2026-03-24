use crate::tabs::Tab;
use eframe::egui;
use std::cell::Cell;
use models::OptimizationResult;
use crate::settings::AppSettings;

mod models;
mod logic;
mod ui;
mod export;

/// Первая лабораторная работа: Методы одномерной оптимизации
pub struct Lab1Tab {
    // Входные параметры
    pub a: f64,
    pub b: f64,
    pub eps: f64,
    pub l: f64,

    // Состояние выбора
    pub selected_func: usize,   // 0: F1, 1: F2
    pub selected_method: usize, // 0: Дихотомия, 1: Золотое сечение, 2: Фибоначчи

    // Результаты
    pub result: Option<OptimizationResult>,
    pub error_msg: Option<String>,
    pub needs_plot_reset: bool,
    pub selected_iteration: Option<usize>,

    // Поля для анимации
    pub is_animating: bool,
    pub current_step: usize,
    pub last_step_time: f64,
    pub animation_speed: f32,
    pub animation_t: f64,
    pub is_smooth_enabled: bool,

    // Поля для экспорта
    pub show_export_dialog: bool,
    pub export_step_interval: usize,
    pub export_start_step: usize,
    pub export_end_step: usize,
    pub export_include_table: bool,
    pub export_mode: usize, // 0 - финал, 1 - шаги
}

impl Default for Lab1Tab {
    fn default() -> Self {
        Self {
            a: -3.0,
            b: 3.0,
            eps: 0.01,
            l: 0.1,
            selected_func: 0,
            selected_method: 0,
            result: None,
            error_msg: None,
            needs_plot_reset: true,
            selected_iteration: None,
            animation_speed: 0.5,
            animation_t: 0.0,
            is_animating: false,
            current_step: 0,
            last_step_time: 0.0,
            is_smooth_enabled: true,
            export_step_interval: 1,
            export_start_step: 1,
            export_end_step: 1,
            show_export_dialog: false,
            export_include_table: true,
            export_mode: 0,
        }
    }
}

impl Lab1Tab {
    /// Математическое определение функций
    pub fn get_f_value(&self, x: f64) -> f64 {
        match self.selected_func {
            0 => 3.0 * x - x.powi(3),                             // F1: 3x - x^3
            1 => (9.0 - x.powi(2)) / (x.powi(2) + 2.0 * x + 3.0), // F2: (9-x^2)/(x^2+2x+3)
            _ => 0.0,
        }
    }

    /// Сбрасывает результаты вычислений и состояние анимации
    pub fn reset_results(&mut self) {
        self.result = None;
        self.current_step = 0;
        self.selected_iteration = None;
        self.is_animating = false;
        self.animation_t = 0.0;
        self.error_msg = None;
    }

    /// Запуск алгоритмов оптимизации
    pub fn run_optimization(&mut self) {
        self.error_msg = None;
        self.selected_iteration = None;

        let f_raw = |x: f64| self.get_f_value(x);
        let f_wrapper = logic::Func {
            f: &f_raw,
            calls: Cell::new(0),
        };

        let is_max = self.selected_func == 0;

        let res = match self.selected_method {
            0 => logic::dichotomy_method(self.a, self.b, self.eps, self.l, &f_wrapper, is_max),
            1 => logic::golden_ratio_method(self.a, self.b, self.eps, self.l, &f_wrapper, is_max),
            2 => logic::fibonacci_method(self.a, self.b, self.eps, self.l, &f_wrapper, is_max),
            _ => unreachable!(),
        };

        match res {
            Ok(data) => self.result = Some(data),
            Err(e) => self.error_msg = Some(e),
        }
    }

    /// Логика управления временем анимации
    pub fn handle_animation_logic(&mut self, ctx: &egui::Context) {
        if !self.is_animating {
            return;
        }

        if let Some(res) = &self.result {
            let now = ctx.input(|i| i.time);
            let elapsed = now - self.last_step_time;
            let step_duration = self.animation_speed as f64;

            if self.current_step >= res.history.len() {
                self.is_animating = false;
                self.animation_t = 1.0;
                return;
            }

            if self.is_smooth_enabled {
                self.animation_t = (elapsed / step_duration).clamp(0.0, 1.0);

                if elapsed >= step_duration {
                    self.current_step = (self.current_step + 1).min(res.history.len());
                    self.last_step_time = now;
                    self.animation_t = 0.0;
                }
            } else {
                self.animation_t = 1.0;

                if elapsed >= step_duration {
                    self.current_step = (self.current_step + 1).min(res.history.len());
                    self.last_step_time = now;
                }
            }

            if self.current_step >= res.history.len() {
                self.is_animating = false;
            }

            ctx.request_repaint();
        }
    }
}

impl Tab for Lab1Tab {
    fn name(&self) -> &str {
        "Лабораторная работа №1"
    }

    fn id(&self) -> usize {
        1
    }

    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.render_ui(ctx, ui);
    }

    fn side_panel(&mut self, ui: &mut egui::Ui) {
        self.side_panel(ui);
    }

    fn update(&mut self, ctx: &egui::Context, settings: &AppSettings) {
        // Синхронизируем настройки анимации
        self.is_smooth_enabled = settings.smooth_animation;
        self.animation_speed = settings.animation_speed;

        self.handle_animation_logic(ctx);
    }

    fn reset(&mut self) {
        self.reset_results();
    }
}
