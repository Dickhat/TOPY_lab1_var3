use crate::app::{OptimizationApp, LabSelection};
use eframe::egui;

impl OptimizationApp {
    pub fn render_ui(&mut self, ctx: &egui::Context) {
        // Верхняя панель с заголовком
        self.render_top_menu(ctx);

        // Левая сворачиваемая панель с элементами управления
        self.render_left_panel(ctx);

        // Центральная область с содержимым текущей вкладки
        self.render_central_content(ctx);
    }

    fn render_top_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu")
            // .default_height(50.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("📚 Методы Оптимизации");
                });
            });
    }

    fn render_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls")
            .resizable(false)
            .default_width(320.0)
            .min_width(320.0)
            .max_width(320.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;
                        ui.add_space(10.0);

                        ui.label(egui::RichText::new("📂 Навигация").strong());

                        ui.group(|ui| {
                            ui.label(egui::RichText::new("📚 Лабораторные работы").strong());

                            if ui
                                .selectable_label(self.selected_lab == LabSelection::Lab1, "Лабораторная работа №1")
                                .clicked()
                            {
                                self.selected_lab = LabSelection::Lab1;
                                self.tab_manager.switch_tab(0);
                            }
                            if ui
                                .selectable_label(self.selected_lab == LabSelection::Lab2, "Лабораторная работа №2")
                                .clicked()
                            {
                                self.selected_lab = LabSelection::Lab2;
                                self.tab_manager.switch_tab(1);
                            }
                        });

                        ui.separator();

                        ui.group(|ui| {
                            ui.label(egui::RichText::new("⚙️ Настройки интерфейса").strong());

                            ui.label("Цветовая схема:");
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.settings.color_theme, crate::settings::ColorTheme::Light, "Светлая");
                                ui.radio_value(&mut self.settings.color_theme, crate::settings::ColorTheme::Dark, "Тёмная");
                                ui.radio_value(&mut self.settings.color_theme, crate::settings::ColorTheme::Auto, "Авто");
                            });

                            ui.checkbox(&mut self.settings.auto_scale_ui, "Автомасштабирование интерфейса");
                        });
                    });
            });

    }

    fn render_central_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.selected_lab {
                LabSelection::None => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(50.0);
                        ui.heading("📚 Выберите лабораторную работу");
                        ui.add_space(20.0);
                        ui.label("Используйте левую панель для выбора метода оптимизации");
                    });
                }
                LabSelection::Lab1 | LabSelection::Lab2 => {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(320.0);
                            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                                if let Some(tab) = self.tab_manager.tabs.get_mut(self.tab_manager.current_tab) {
                                    tab.side_panel(ui);
                                }
                            });
                        });

                        ui.separator();

                        ui.vertical(|ui| {
                            ui.set_width(ui.available_size().x - 16.0);
                            if let Some(tab) = self.tab_manager.tabs.get_mut(self.tab_manager.current_tab) {
                                tab.ui(ctx, ui);
                            }
                        });
                    });
                }
            }
        });
    }
}
