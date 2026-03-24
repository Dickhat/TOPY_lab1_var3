use eframe::egui;
use crate::settings::AppSettings;

pub mod lab1;
pub mod lab2;

/// Трейт для определения поведения каждой лабораторной работы
pub trait Tab {
    /// Имя вкладки (отображается в меню)
    fn name(&self) -> &str;

    /// Уникальный идентификатор вкладки
    fn id(&self) -> usize;

    /// Рисует UI вкладки (центральная часть)
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);

    /// Рисует панель настроек своей вкладки (левый сайдбар)
    fn side_panel(&mut self, ui: &mut egui::Ui);

    /// Обновляет логику (вызывается каждый кадр)
    fn update(&mut self, ctx: &egui::Context, settings: &AppSettings);

    /// Сброс состояния вкладки
    fn reset(&mut self);
}

/// Менеджер для управления всеми табами
pub struct TabManager {
    pub tabs: Vec<Box<dyn Tab>>,
    pub current_tab: usize,
}

impl TabManager {
    pub fn new() -> Self {
        let tabs: Vec<Box<dyn Tab>> = vec![
            Box::new(lab1::Lab1Tab::default()),
            Box::new(lab2::Lab2Tab::default()),
        ];

        Self {
            tabs,
            current_tab: 0,
        }
    }

    pub fn current_tab(&mut self) -> &mut Box<dyn Tab> {
        &mut self.tabs[self.current_tab]
    }

    pub fn switch_tab(&mut self, tab_id: usize) {
        if tab_id < self.tabs.len() {
            self.current_tab = tab_id;
        }
    }
}
