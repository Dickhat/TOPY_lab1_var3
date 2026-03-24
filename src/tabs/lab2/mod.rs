use crate::tabs::Tab;
use eframe::egui;
use crate::settings::AppSettings;

mod models;
use models::HJResult;
mod logic;
mod ui;

/// Вторая лабораторная работа: Метод Хука-Джевса с дискретным шагом
pub struct Lab2Tab {
    // Параметры
    pub selected_func: usize,      // 0: F1(x1,x2), 1: F2(x1,x2,x3)
    pub epsilon: f64,              // Точность поиска
    pub delta_init: f64,           // Начальный дискретный шаг
    pub delta_min: f64,            // Минимальный шаг
    pub use_custom_start: bool,    // Использовать пользовательскую начальную точку
    pub start_point: Vec<f64>,     // Начальная точка

    // Результаты
    pub result: Option<HJResult>,
    pub error_msg: Option<String>,
    pub selected_iteration: Option<usize>,

    // Для анимации
    pub is_animating: bool,
    pub current_step: usize,
    pub animation_speed: f32,
    pub animation_t: f64,
    pub last_step_time: f64,
    pub is_smooth_enabled: bool,

    // Экспорт
    pub show_export_dialog: bool,
}

impl Default for Lab2Tab {
    fn default() -> Self {
        Self {
            selected_func: 0,
            epsilon: 0.01,
            delta_init: 1.0,
            delta_min: 0.001,
            use_custom_start: false,
            start_point: vec![1.0, 1.0],
            result: None,
            error_msg: None,
            selected_iteration: None,
            is_animating: false,
            current_step: 0,
            animation_speed: 0.5,
            animation_t: 0.0,
            last_step_time: 0.0,
            is_smooth_enabled: true,
            show_export_dialog: false,
        }
    }
}

impl Lab2Tab {
    /// Какова размерность задачи
    pub fn dimension(&self) -> usize {
        match self.selected_func {
            0 => 2,  // F1(x1, x2)
            1 => 3,  // F2(x1, x2, x3)
            _ => 2,
        }
    }

    /// Получить значение функции в точке
    pub fn evaluate(&self, point: &[f64]) -> f64 {
        match self.selected_func {
            0 => {
                // F1(x) = -6*x1 - 4*x2 + x1^2 + x2^2 + 18
                let x1 = point[0];
                let x2 = point[1];
                -6.0 * x1 - 4.0 * x2 + x1 * x1 + x2 * x2 + 18.0
            }
            1 => {
                // F2(x) = 4*x1^2 + 3*x2^2 + x3^2 + 4*x1*x2 - 2*x2*x3 - 16*x1 - 4*x3
                let x1 = point[0];
                let x2 = point[1];
                let x3 = point[2];
                4.0 * x1 * x1 + 3.0 * x2 * x2 + x3 * x3 + 4.0 * x1 * x2
                    - 2.0 * x2 * x3 - 16.0 * x1 - 4.0 * x3
            }
            _ => 0.0,
        }
    }

    /// Функция описание
    pub fn func_name(&self) -> &str {
        match self.selected_func {
            0 => "F1: -6x₁ - 4x₂ + x₁² + x₂² + 18",
            1 => "F2: 4x₁² + 3x₂² + x₃² + 4x₁x₂ - 2x₂x₃ - 16x₁ - 4x₃",
            _ => "Unknown",
        }
    }

    /// Сбросить результаты
    pub fn reset_results(&mut self) {
        self.result = None;
        self.current_step = 0;
        self.selected_iteration = None;
        self.is_animating = false;
        self.animation_t = 0.0;
        self.error_msg = None;
    }

    /// Запустить оптимизацию методом Хука-Джевса
    pub fn run_optimization(&mut self) {
        self.error_msg = None;
        self.selected_iteration = None;

        // Проверяем начальную точку
        if self.start_point.len() != self.dimension() {
            self.start_point = vec![1.0; self.dimension()];
        }

        // Запускаем метод
        let result = logic::hooke_jeeves(
            self.start_point.clone(),
            self.delta_init,
            self.delta_min,
            self.epsilon,
            |point| self.evaluate(point),
        );

        match result {
            Ok(res) => self.result = Some(res),
            Err(e) => self.error_msg = Some(e),
        }
    }

    /// Логика управления временем анимации
    pub fn handle_animation_logic(&mut self, ctx: &egui::Context) {
        if self.is_animating {
            if let Some(ref res) = self.result {
                let now = ctx.input(|i| i.time);
                let elapsed = now - self.last_step_time;

                self.animation_t = (elapsed / self.animation_speed as f64).min(1.0);

                if self.animation_t >= 1.0 {
                    if self.current_step < res.iterations.len() {
                        self.current_step += 1;
                        self.last_step_time = now;
                        self.animation_t = 0.0;
                    } else {
                        self.is_animating = false;
                    }
                }
                ctx.request_repaint();
            }
        } else {
            self.last_step_time =
                ctx.input(|i| i.time) - (self.animation_t * self.animation_speed as f64);
        }
    }
}

impl Tab for Lab2Tab {
    fn name(&self) -> &str {
        "Лабораторная работа №2"
    }

    fn id(&self) -> usize {
        2
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
