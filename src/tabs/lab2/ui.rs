use super::Lab2Tab;
use eframe::egui;

impl Lab2Tab {
    pub fn render_ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Метод Хука-Джевса с дискретным шагом");

            if let Some(result) = &self.result {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("📋 Итерации").strong());

                    egui::ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("hj_iterations")
                            .striped(true)
                            .show(ui, |ui| {
                                // Заголовки
                                ui.label(egui::RichText::new("Итерация").strong());
                                ui.label(egui::RichText::new("Точка").strong());
                                ui.label(egui::RichText::new("f(x)").strong());
                                ui.label(egui::RichText::new("Шаг Δ").strong());
                                ui.end_row();

                                // Данные
                                for iter_data in &result.iterations {
                                    ui.label(&format!("{}", iter_data.iteration));
                                    ui.label(&format!("{:?}", iter_data.current_point));
                                    ui.label(&format!("{:.6}", iter_data.f_value));
                                    ui.label(&format!("{:.6}", iter_data.delta));
                                    ui.end_row();
                                }
                            });
                    });
                });
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(50.0);
                    ui.label("Нажмите 'Запустить оптимизацию' для начала расчета");
                });
            }
        });
    }

    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.y = 10.0;
        ui.add_space(10.0);

        ui.vertical_centered(|ui| {
            ui.heading("🔶 Метод Хука-Джевса");
        });
        ui.separator();

        // Выбор функции
        ui.group(|ui| {
            ui.label(egui::RichText::new("🎯 Функция").strong());
            ui.radio_value(&mut self.selected_func, 0, "F1: 2 переменные");
            ui.radio_value(&mut self.selected_func, 1, "F2: 3 переменные");
        });

        // Параметры
        ui.group(|ui| {
            ui.label(egui::RichText::new("📝 Параметры").strong());

            ui.label("Точность ε:");
            ui.add(egui::DragValue::new(&mut self.epsilon).speed(0.001));

            ui.label("Начальный шаг Δ:");
            ui.add(egui::DragValue::new(&mut self.delta_init).speed(0.1));

            ui.label("Минимальный шаг Δ_min:");
            ui.add(egui::DragValue::new(&mut self.delta_min).speed(0.001));
        });

        // Начальная точка
        ui.group(|ui| {
            ui.label(egui::RichText::new("📍 Начальная точка").strong());
            ui.checkbox(&mut self.use_custom_start, "Задать вручную");

            if self.use_custom_start {
                let dim = self.dimension();
                for i in 0..dim {
                    ui.horizontal(|ui| {
                        ui.label(format!("x{}:", i + 1));
                        if i < self.start_point.len() {
                            ui.add(egui::DragValue::new(&mut self.start_point[i]).speed(0.1));
                        }
                    });
                }
            } else {
                ui.label("Используется точка (1, 1, ...) по умолчанию");
            }
        });

        // Кнопка расчета
        ui.vertical_centered(|ui| {
            let button = egui::Button::new(egui::RichText::new("🚀 Запустить оптимизацию").size(16.0));
            if ui.add(button).clicked() {
                self.run_optimization();
            }
        });

        // Результаты
        if let Some(result) = &self.result {
            ui.group(|ui| {
                ui.label(egui::RichText::new("📊 Результаты").strong());
                ui.label(format!("Оптимальная точка: {:?}", result.x_opt));
                ui.label(format!("Значение функции: {:.6}", result.f_opt));
                ui.label(format!("Итераций: {}", result.num_iterations));
                ui.label(format!("Вызовов функции: {}", result.function_calls));
            });
        }

        // Ошибки
        if let Some(err) = &self.error_msg {
            ui.group(|ui| {
                ui.colored_label(egui::Color32::RED, "❌ Ошибка:");
                ui.colored_label(egui::Color32::RED, err);
            });
        }

        // Заполняем оставшееся пространство
        ui.allocate_space(egui::Vec2::new(0.0, ui.available_height()));
    }
}
