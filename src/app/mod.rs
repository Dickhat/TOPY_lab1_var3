pub mod ui;

use crate::tabs::TabManager;
use crate::settings::AppSettings;
use eframe::egui;

/// Состояние выбора лабораторной работы
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LabSelection {
    None,      // Ничего не выбрано
    Lab1,      // Выбрана первая работа
    Lab2,      // Выбрана вторая работа
}

/// Главная структура приложения с поддержкой вкладок
pub struct OptimizationApp {
    pub tab_manager: TabManager,
    pub settings: AppSettings,
    pub selected_lab: LabSelection,
}

impl Default for OptimizationApp {
    fn default() -> Self {
        Self {
            tab_manager: TabManager::new(),
            settings: AppSettings::default(),
            selected_lab: LabSelection::None,
        }
    }
}

impl eframe::App for OptimizationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Применяем глобальные настройки
        self.settings.apply_theme(ctx);

        // 2. Обновляем текущую вкладку
        if let Some(tab) = self.tab_manager.tabs.get_mut(self.tab_manager.current_tab) {
            tab.update(ctx, &self.settings);
        }

        // 3. Вызов функций отрисовки из ui.rs
        self.render_ui(ctx);
    }
}