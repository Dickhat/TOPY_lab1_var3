use eframe::egui;

/// Глобальные настройки приложения
#[derive(Clone)]
pub struct AppSettings {
    pub color_theme: ColorTheme,
    pub animation_speed: f32,
    pub smooth_animation: bool,
    pub auto_scale_ui: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorTheme {
    Light,
    Dark,
    Auto,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_theme: ColorTheme::Light,
            animation_speed: 0.5,
            smooth_animation: true,
            auto_scale_ui: true,
        }
    }
}

impl AppSettings {
    /// Применить текущую цветовую схему к контексту egui
    pub fn apply_theme(&self, ctx: &egui::Context) {
        match self.color_theme {
            ColorTheme::Light => ctx.set_visuals(egui::Visuals::light()),
            ColorTheme::Dark => ctx.set_visuals(egui::Visuals::dark()),
            ColorTheme::Auto => {
                // TODO: Определить текущую тему системы
                ctx.set_visuals(egui::Visuals::light());
            }
        }

        if self.auto_scale_ui {
            ctx.set_pixels_per_point(1.1);
        }
    }

    /// Рисует панель настроек
    pub fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("⚙️ Общие настройки").strong());
            ui.separator();

            // Цветовая схема
            ui.label("Цветовая схема:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.color_theme, ColorTheme::Light, "Светлая");
                ui.radio_value(&mut self.color_theme, ColorTheme::Dark, "Тёмная");
                ui.radio_value(&mut self.color_theme, ColorTheme::Auto, "Авто");
            });

            // Анимация
            ui.checkbox(&mut self.smooth_animation, "Плавная анимация");
            ui.label("Скорость анимации:");
            ui.add(egui::Slider::new(&mut self.animation_speed, 0.1..=2.0).text("скорость"));

            // UI масштабирование
            ui.checkbox(&mut self.auto_scale_ui, "Автомасштабирование интерфейса");
        });
    }
}
