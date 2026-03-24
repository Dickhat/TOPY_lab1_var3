use super::Lab1Tab;
use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints, Points, VLine};

impl Lab1Tab {
    pub fn render_ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // График и визуализация результата
            self.render_plot(ui);

            // Кнопка сброса масштаба (под графиком)
            if ui.button("🔄 Сбросить масштаб").clicked() {
                self.needs_plot_reset = true;
            }

            // Таблица результатов
            self.render_results_table(ui);
        });

        // Диалог экспорта (если будет реализован)
        self.render_export_dialog(_ctx);
    }

    pub fn side_panel(&mut self, ui: &mut egui::Ui) {
        // --- ЛОГИКА ВАЛИДАЦИИ ---
        let is_eps_valid = self.eps > 0.0;
        let is_l_valid = self.l > 0.0;
        let is_interval_valid = self.b >= self.a;
        let is_eps_l_valid = 2.0 * self.eps < self.l && is_eps_valid && is_l_valid;
        let can_calculate = is_interval_valid && is_eps_l_valid;

        // Добавляем область прокрутки
        // egui::ScrollArea::vertical()
        //     .auto_shrink([false, false]) // Растягиваться на всю высоту панели
        //     .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    ui.heading("📊 Оптимизация");
                });
                ui.separator();

                        // БЛОК 1: ВЫБОР ФУНКЦИИ
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("🎯 Функция").strong());
                            if ui
                                .radio_value(&mut self.selected_func, 0, "F1: 3x - x³ (Max)")
                                .changed()
                            {
                                self.reset_results();
                            }
                            if ui
                                .radio_value(
                                    &mut self.selected_func,
                                    1,
                                    "F2: (9-x²)/(x²+2x+3) (Min)",
                                )
                                .changed()
                            {
                                self.reset_results();
                            }
                        });

                        // БЛОК 2: МЕТОД
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("🛠 Метод").strong());
                            let combo = egui::ComboBox::from_label("")
                                .selected_text(match self.selected_method {
                                    0 => "Дихотомия",
                                    1 => "Золотое сечение",
                                    _ => "Фибоначчи",
                                })
                                .show_ui(ui, |ui| {
                                    let mut changed = false;
                                    changed |= ui
                                        .selectable_value(&mut self.selected_method, 0, "Дихотомия")
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut self.selected_method,
                                            1,
                                            "Золотое сечение",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(&mut self.selected_method, 2, "Фибоначчи")
                                        .changed();
                                    changed
                                });

                            // Если в выпадающем списке что-то выбрали
                            if combo.inner.unwrap_or(false) {
                                self.reset_results();
                            }
                        });

                        // БЛОК 3: ПАРАМЕТРЫ
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("📝 Параметры").strong());
                            egui::Grid::new("inputs_grid")
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    // Валидация интервала [a, b]
                                    let a_label = if is_interval_valid {
                                        egui::RichText::new("Начало a:")
                                    } else {
                                        egui::RichText::new("Начало a:").color(egui::Color32::RED)
                                    };
                                    ui.label(a_label);
                                    if ui
                                        .add(egui::DragValue::new(&mut self.a).speed(0.1))
                                        .changed()
                                    {
                                        self.needs_plot_reset = true;
                                        self.reset_results();
                                    }
                                    ui.end_row();

                                    let b_label = if is_interval_valid {
                                        egui::RichText::new("Конец b:")
                                    } else {
                                        egui::RichText::new("Конец b:").color(egui::Color32::RED)
                                    };
                                    ui.label(b_label);
                                    if ui
                                        .add(egui::DragValue::new(&mut self.b).speed(0.1))
                                        .changed()
                                    {
                                        self.needs_plot_reset = true;
                                        self.reset_results();
                                    }
                                    ui.end_row();

                                    // Валидация 2*eps < l
                                    let eps_label = if is_eps_l_valid {
                                        egui::RichText::new("Точность ε:")
                                    } else {
                                        egui::RichText::new("Точность ε:").color(egui::Color32::RED)
                                    };
                                    ui.label(eps_label);
                                    if ui
                                        .add(egui::DragValue::new(&mut self.eps).speed(0.001))
                                        .changed()
                                    {
                                        self.reset_results();
                                    }
                                    ui.end_row();

                                    let l_label = if is_eps_l_valid {
                                        egui::RichText::new("Длина l:")
                                    } else {
                                        egui::RichText::new("Длина l:").color(egui::Color32::RED)
                                    };
                                    ui.label(l_label);
                                    if ui
                                        .add(egui::DragValue::new(&mut self.l).speed(0.001))
                                        .changed()
                                    {
                                        self.reset_results();
                                    }
                                    ui.end_row();
                                });

                            // Вывод сообщений об ошибках под полями ввода
                            if !can_calculate {
                                ui.add_space(5.0);
                                if !is_interval_valid {
                                    ui.colored_label(egui::Color32::RED, "⚠ Ошибка: a > b");
                                }
                                if !is_eps_l_valid {
                                    ui.colored_label(
                                        egui::Color32::RED,
                                        "⚠ Ошибка: 2ε должен быть < l \n(и ε, и l должны быть > 0)",
                                    );
                                }
                            }
                        });

                        // БЛОК 4: КНОПКА РАСЧЕТА
                        // КНОПКА РАССЧИТАТЬ (Блокируется при ошибках)
                        ui.vertical_centered(|ui| {
                            ui.add_enabled_ui(can_calculate, |ui| {
                                let btn = egui::Button::new(
                                    egui::RichText::new("🚀 Рассчитать").size(16.0),
                                )
                                .min_size(egui::vec2(160.0, 32.0));

                                if ui.add(btn).clicked() {
                                    self.run_optimization();
                                    self.needs_plot_reset = true;
                                    if let Some(res) = &self.result {
                                        self.current_step = res.history.len();
                                        self.is_animating = false;
                                    }
                                }
                            });

                            if !can_calculate {
                                ui.label(
                                    egui::RichText::new("Исправьте параметры").weak().size(10.0),
                                );
                            }
                        });

                        ui.group(|ui| {
                            ui.label(egui::RichText::new("🎬 Анимация").strong());
                            ui.checkbox(&mut self.is_smooth_enabled, "Плавное перемещение");
                            ui.add(
                                egui::Slider::new(&mut self.animation_speed, 0.1..=3.0)
                                    .text("сек/шаг"),
                            );

                            ui.horizontal(|ui| {
                                let play_btn_text = if self.is_animating {
                                    "⏸ Пауза"
                                } else {
                                    "▶ Старт"
                                };
                                if ui.button(play_btn_text).clicked() {
                                    self.selected_iteration = None;
                                    if self.result.is_none() {
                                        self.run_optimization();
                                    }

                                    // Если мы дошли до конца и нажимаем "Старт" снова — только тогда сбрасываем на 0
                                    if let Some(res) = &self.result {
                                        if !self.is_animating
                                            && self.current_step >= res.history.len()
                                        {
                                            self.current_step = 0;
                                            self.animation_t = 0.0;
                                        }
                                    }

                                    self.is_animating = !self.is_animating;
                                }

                                // Кнопка "Шаг вперед"
                                if ui.button("⏭ Шаг").clicked() {
                                    if let Some(res) = &self.result {
                                        if self.current_step < res.history.len() {
                                            self.is_animating = false;
                                            self.current_step += 1;
                                            self.animation_t = 0.0; // При ручном шаге прыгаем мгновенно
                                            self.selected_iteration = Some(self.current_step - 1);
                                        }
                                    }
                                }

                                if ui.button("🔄 Сброс").clicked() {
                                    self.is_animating = false;
                                    self.current_step = 0;
                                    self.selected_iteration = None;
                                    self.animation_t = 0.0;
                                }
                            });

                            if let Some(res) = &self.result {
                                ui.add_space(5.0);
                                ui.separator();
                                if ui
                                    .add(
                                        egui::Slider::new(
                                            &mut self.current_step,
                                            0..=res.history.len(),
                                        )
                                        .text("Тек. шаг"),
                                    )
                                    .changed()
                                {
                                    self.is_animating = false;
                                    self.selected_iteration = if self.current_step > 0 {
                                        Some(self.current_step - 1)
                                    } else {
                                        None
                                    };
                                }
                            }
                        });

                        // БЛОК 5: РЕЗУЛЬТАТЫ
                        if let Some(res) = &self.result {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new("📊 Результаты").strong());
                                ui.label(format!("Оптимальное значение: {:.6}", res.x_opt));
                                ui.label(format!("Значение функции: {:.6}", res.f_opt));
                                ui.label(format!("Количество итераций: {}", res.history.len()));
                                ui.label(format!("Вызовов функции: {}", res.fn_calls));
                            });
                        }

                        // БЛОК 6: ОШИБКИ
                        if let Some(err) = &self.error_msg {
                            ui.group(|ui| {
                                ui.colored_label(egui::Color32::RED, "❌ Ошибка:");
                                ui.colored_label(egui::Color32::RED, err);
                            });
                        }

        // Заполняем оставшееся пространство
        ui.allocate_space(egui::Vec2::new(0.0, ui.available_height()));
    }


    pub fn render_plot(&mut self, ui: &mut egui::Ui) {
        // Используем разный id если нужно сбросить масштаб
        let plot_id = if self.needs_plot_reset {
            self.needs_plot_reset = false;
            format!("optimization_plot_reset_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis())
        } else {
            "optimization_plot".to_string()
        };

        let plot = Plot::new(plot_id)
            .legend(Legend::default())
            .allow_drag(true)
            .allow_zoom(true)
            .allow_scroll(true)
            .width(600.0)
            .height(400.0);

        plot.show(ui, |plot_ui| {
            // График функции
            let n_points = 200;
            let x_min = self.a - 1.0;
            let x_max = self.b + 1.0;
            let dx = (x_max - x_min) / (n_points as f64 - 1.0);

            let points: PlotPoints = (0..n_points)
                .map(|i| {
                    let x = x_min + i as f64 * dx;
                    let y = self.get_f_value(x);
                    [x, y]
                })
                .collect();

            plot_ui.line(Line::new(points).name("Функция"));

            // Текущий интервал поиска
            if let Some(res) = &self.result {
                let step_idx = if self.current_step == 0 {
                    0
                } else {
                    (self.current_step - 1).min(res.history.len().saturating_sub(1))
                };

                if step_idx < res.history.len() {
                    let use_interp = self.is_animating && self.is_smooth_enabled && self.current_step < res.history.len();
                    let (a, b, lambda, mu, f_lambda, f_mu) = if use_interp && self.current_step < res.history.len() - 1 {
                        let current = &res.history[self.current_step];
                        let next = &res.history[self.current_step + 1];
                        let t = self.animation_t;

                        (
                            current.a + (next.a - current.a) * t,
                            current.b + (next.b - current.b) * t,
                            current.lambda + (next.lambda - current.lambda) * t,
                            current.mu + (next.mu - current.mu) * t,
                            current.f_lambda + (next.f_lambda - current.f_lambda) * t,
                            current.f_mu + (next.f_mu - current.f_mu) * t,
                        )
                    } else {
                        let iter = &res.history[step_idx];
                        (iter.a, iter.b, iter.lambda, iter.mu, iter.f_lambda, iter.f_mu)
                    };

                    // Интервал [a, b]
                    plot_ui.vline(VLine::new(a).name("a"));
                    plot_ui.vline(VLine::new(b).name("b"));

                    // Точки lambda и mu
                    plot_ui.points(
                        Points::new(vec![[lambda, f_lambda]])
                            .name("λ")
                            .color(egui::Color32::RED)
                            .radius(5.0),
                    );
                    plot_ui.points(
                        Points::new(vec![[mu, f_mu]])
                            .name("μ")
                            .color(egui::Color32::BLUE)
                            .radius(5.0),
                    );
                }
            }
        });
    }

    pub fn render_results_table(&mut self, ui: &mut egui::Ui) {
        if let Some(res) = &self.result {
            ui.group(|ui| {
                ui.label(egui::RichText::new("📋 Итерации").strong());

                egui::ScrollArea::horizontal().show(ui, |ui| {
                    // Заголовки
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("k").strong());
                        ui.label(egui::RichText::new("a").strong());
                        ui.label(egui::RichText::new("b").strong());
                        ui.label(egui::RichText::new("λ").strong());
                        ui.label(egui::RichText::new("μ").strong());
                        ui.label(egui::RichText::new("f(λ)").strong());
                        ui.label(egui::RichText::new("f(μ)").strong());
                    });

                    ui.separator();

                    // Данные
                    for (i, iter) in res.history.iter().enumerate() {
                        let is_selected = self.selected_iteration == Some(i);

                        let row = ui.horizontal(|ui| {
                            if is_selected {
                                ui.colored_label(egui::Color32::YELLOW, &format!("{}", iter.k));
                            } else {
                                ui.label(&format!("{}", iter.k));
                            }
                            ui.label(&format!("{:.6}", iter.a));
                            ui.label(&format!("{:.6}", iter.b));
                            ui.label(&format!("{:.6}", iter.lambda));
                            ui.label(&format!("{:.6}", iter.mu));
                            ui.label(&format!("{:.6}", iter.f_lambda));
                            ui.label(&format!("{:.6}", iter.f_mu));
                        });

                        if row.response.clicked() {
                            self.selected_iteration = Some(i);
                            self.current_step = i + 1;
                            self.animation_t = 1.0; // Мгновенный переход к выбранной итерации
                            self.is_animating = false;
                        }
                    }
                });
            });
        }
    }

    pub fn render_export_dialog(&mut self, ctx: &egui::Context) {
        // TODO: Реализовать диалог экспорта
    }
}
