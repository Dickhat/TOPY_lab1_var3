use crate::app::OptimizationApp;
use crate::utils::lerp; // Теперь это заработает, так как мы добавили mod utils в main.rs
use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints, Points, VLine};

impl OptimizationApp {
    pub fn render_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(280.0)
            .min_width(250.0)
            .show(ctx, |ui| {
                // Добавляем область прокрутки
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false]) // Растягиваться на всю высоту панели
                    .show(ui, |ui| {
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
                                    ui.label("Начало a:");
                                    if ui
                                        .add(egui::DragValue::new(&mut self.a).speed(0.1))
                                        .changed()
                                    {
                                        self.needs_plot_reset = true;
                                        self.reset_results(); // Сброс при изменении 'a'
                                    }
                                    ui.end_row();

                                    ui.label("Конец b:");
                                    if ui
                                        .add(egui::DragValue::new(&mut self.b).speed(0.1))
                                        .changed()
                                    {
                                        self.needs_plot_reset = true;
                                        self.reset_results(); // Сброс при изменении 'b'
                                    }
                                    ui.end_row();

                                    ui.label("Точность ε:");
                                    if ui
                                        .add(egui::DragValue::new(&mut self.eps).speed(0.001))
                                        .changed()
                                    {
                                        self.reset_results(); // Сброс при изменении 'eps'
                                    }
                                    ui.end_row();

                                    ui.label("Длина l:");
                                    if ui
                                        .add(egui::DragValue::new(&mut self.l).speed(0.001))
                                        .changed()
                                    {
                                        self.reset_results(); // Сброс при изменении 'l'
                                    }
                                    ui.end_row();
                                });
                        });

                        // БЛОК 4: КНОПКА РАСЧЕТА
                        ui.vertical_centered(|ui| {
                            let btn =
                                egui::Button::new(egui::RichText::new("🚀 Рассчитать").size(16.0))
                                    .min_size(egui::vec2(160.0, 32.0));

                            if ui.add(btn).clicked() {
                                self.run_optimization();
                                self.needs_plot_reset = true;
                                if let Some(res) = &self.result {
                                    self.current_step = res.history.len();
                                    self.is_animating = false;
                                    self.animation_t = 0.0;
                                }
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

                        if self.result.is_some() {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Экспорт:");
                                if ui.button("📄 TXT").clicked() {
                                    self.save_report();
                                }
                                if ui.button("📝 .DOCX").clicked() {
                                    self.show_export_dialog = true;
                                    if let Some(res) = &self.result {
                                        self.export_start_step = 1;
                                        self.export_end_step = res.history.len();
                                    }
                                }
                            });
                        }

                        if let Some(res) = &self.result {
                            ui.add_space(10.0);
                            let accent_color = egui::Color32::from_rgb(255, 215, 0);
                            egui::Frame::group(ui.style())
                                .fill(ui.visuals().window_fill())
                                .stroke(egui::Stroke::new(2.0, accent_color))
                                .rounding(8.0)
                                .inner_margin(12.0)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(
                                            egui::RichText::new("🏁 ИТОГОВЫЙ ОТВЕТ")
                                                .strong()
                                                .color(accent_color)
                                                .size(16.0),
                                        );
                                    });
                                    ui.add_space(5.0);
                                    ui.separator();
                                    // Используем сетку для выравнивания подписей и значений
                                    egui::Grid::new("final_results_grid")
                                        .num_columns(2)
                                        .spacing([15.0, 12.0]) // Больше отступов для читаемости
                                        .show(ui, |ui| {
                                            // 1. Оптимальное значение аргумента
                                            ui.label(
                                                egui::RichText::new("Аргумент x*:").size(14.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("{:.6}", res.x_opt))
                                                    .strong()
                                                    .color(egui::Color32::LIGHT_GREEN)
                                                    .size(16.0),
                                            );
                                            ui.end_row();

                                            // 2. Оптимальное значение функции
                                            ui.label(
                                                egui::RichText::new("Функция f(x*):").size(14.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(format!("{:.6}", res.f_opt))
                                                    .strong()
                                                    .color(egui::Color32::LIGHT_BLUE)
                                                    .size(16.0),
                                            );
                                            ui.end_row();

                                            // 3. Количество вычислений функции (Требование задания №4)
                                            ui.label(
                                                egui::RichText::new("Вычислений f:").size(14.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(res.fn_calls.to_string())
                                                    .strong()
                                                    .color(egui::Color32::LIGHT_RED)
                                                    .size(16.0),
                                            );
                                            ui.end_row();
                                            // 3. Количество итераций
                                            ui.label(
                                                egui::RichText::new("Всего итераций:").size(14.0),
                                            );
                                            ui.label(
                                                egui::RichText::new(res.history.len().to_string())
                                                    .strong()
                                                    .size(16.0)
                                                    .color(egui::Color32::LIGHT_BLUE),
                                            );
                                            ui.end_row();
                                        });
                                });
                        }
                    })
            });
    }

    pub fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("table_panel")
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("📊 Таблица итераций (нажмите на строку для просмотра)");
                ui.separator();

                if let Some(res) = &self.result {
                    let col_width = (ui.available_width() - 30.0) / 7.0;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Grid::new("it_grid")
                                .striped(true)
                                .min_col_width(col_width)
                                .max_col_width(col_width)
                                .show(ui, |ui| {
                                    // Заголовки
                                    ui.vertical_centered(|ui| ui.strong("K"));
                                    ui.vertical_centered(|ui| ui.strong("a_k"));
                                    ui.vertical_centered(|ui| ui.strong("b_k"));
                                    ui.vertical_centered(|ui| ui.strong("λ"));
                                    ui.vertical_centered(|ui| ui.strong("μ"));
                                    ui.vertical_centered(|ui| ui.strong("F(λ)"));
                                    ui.vertical_centered(|ui| ui.strong("F(μ)"));
                                    ui.end_row();

                                    // Строки данных
                                    for (i, it) in
                                        res.history.iter().take(self.current_step).enumerate()
                                    {
                                        let is_selected = self.selected_iteration == Some(i);

                                        // Функция-помощник для создания кликабельной ячейки
                                        let mut make_cell = |text: String| {
                                            let resp = ui
                                                .add(egui::SelectableLabel::new(is_selected, text));
                                            if resp.clicked() {
                                                if is_selected {
                                                    self.selected_iteration = None;
                                                } else {
                                                    self.selected_iteration = Some(i);
                                                    self.is_animating = false; // Останавливаем анимацию при ручном выборе
                                                }
                                            }
                                        };

                                        make_cell(it.k.to_string());
                                        make_cell(format!("{:.4}", it.a));
                                        make_cell(format!("{:.4}", it.b));
                                        make_cell(format!("{:.4}", it.lambda));
                                        make_cell(format!("{:.4}", it.mu));
                                        make_cell(format!("{:.4}", it.f_lambda));
                                        make_cell(format!("{:.4}", it.f_mu));
                                        ui.end_row();
                                    }
                                });
                        });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Данные отсутствуют. Нажмите 'Рассчитать'.");
                    });
                }
            });
    }

    pub fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📈 График функции и интервалы");

            let plot = Plot::new("Optimization Plot")
                .legend(Legend::default())
                .show_axes([true, true])
                .data_aspect(1.0)
                .allow_zoom(true)
                .allow_drag(true);

            plot.show(ui, |plot_ui| {
                if self.needs_plot_reset {
                    let margin = (self.b - self.a).abs() * 0.5;
                    plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                        [(self.a - margin), -6.0],
                        [(self.b + margin), 6.0],
                    ));
                    self.needs_plot_reset = false;
                }

                // 1. ЦЕЛЕВАЯ ФУНКЦИЯ
                let span = 50.0;
                let center_x = (self.a + self.b) / 2.0;
                let points: PlotPoints = (0..=2000)
                    .map(|i| {
                        let x = (center_x - span) + (i as f64 / 2000.0) * (span * 2.0);
                        [x, self.get_f_value(x)]
                    })
                    .collect();

                plot_ui.line(
                    Line::new(points)
                        .name("f(x)")
                        .color(egui::Color32::LIGHT_BLUE),
                );

                // 2. СТАРТОВЫЕ ГРАНИЦЫ (Синие)
                plot_ui.vline(
                    VLine::new(self.a)
                        .color(egui::Color32::BLUE)
                        .name("Старт a"),
                );
                plot_ui.vline(
                    VLine::new(self.b)
                        .color(egui::Color32::BLUE)
                        .name("Старт b"),
                );

                if let Some(res) = &self.result {
                    // Если плавность выключена или мы не анимируем активно, фиксируем t на 1.0 (конечная точка шага)
                    let drawing_t = if self.is_smooth_enabled {
                        self.animation_t
                    } else {
                        0.0
                    };
                    let (ca, cb, cl, cm);

                    // ПРИОРИТЕТ 1: Если пользователь кликнул в таблице
                    if let Some(idx) = self.selected_iteration {
                        let it = &res.history[idx];
                        ca = it.a;
                        cb = it.b;
                        cl = it.lambda;
                        cm = it.mu;
                    }
                    // ПРИОРИТЕТ 2: Если запущена анимация
                    else if self.current_step < res.history.len() {
                        let next = &res.history[self.current_step];
                        let (pa, pb, pl, pm) = if self.current_step == 0 {
                            (self.a, self.b, self.a, self.b)
                        } else {
                            let prev = &res.history[self.current_step - 1];
                            (prev.a, prev.b, prev.lambda, prev.mu)
                        };
                        ca = lerp(pa, next.a, drawing_t);
                        cb = lerp(pb, next.b, drawing_t);
                        cl = lerp(pl, next.lambda, drawing_t);
                        cm = lerp(pm, next.mu, drawing_t);
                    }
                    // ПРИОРИТЕТ 3: Состояние паузы или завершения
                    else {
                        let last = res.history.last().unwrap();
                        ca = last.a;
                        cb = last.b;
                        cl = last.lambda;
                        cm = last.mu;
                    }

                    // Рисуем оранжевые линии и точки
                    let orange = egui::Color32::from_rgb(255, 165, 0);
                    plot_ui.vline(VLine::new(ca).color(orange).width(2.5).name("a_k"));
                    plot_ui.vline(VLine::new(cb).color(orange).width(2.5).name("b_k"));

                    // Точки lambda и mu (рисуем их на кривой функции)
                    plot_ui.points(
                        Points::new(vec![[cl, self.get_f_value(cl)], [cm, self.get_f_value(cm)]])
                            .color(orange)
                            .radius(5.0),
                    );

                    // --- 4. ТОЧКА λ (Лямбда) ---
                    // Используем насыщенный оранжево-красный цвет
                    let lambda_color = egui::Color32::from_rgb(255, 100, 0);
                    plot_ui.points(
                        Points::new(vec![[cl, self.get_f_value(cl)]])
                            .name("λ")
                            .color(lambda_color)
                            .radius(6.0)
                            .shape(egui_plot::MarkerShape::Circle),
                    );

                    // --- 5. ТОЧКА μ (Мю) ---
                    // Используем сине-зеленый (бирюзовый) цвет для максимального контраста с лямбдой
                    let mu_color = egui::Color32::from_rgb(0, 180, 180);
                    plot_ui.points(
                        Points::new(vec![[cm, self.get_f_value(cm)]])
                            .name("μ")
                            .color(mu_color)
                            .radius(6.0)
                            .shape(egui_plot::MarkerShape::Circle),
                    );

                    // Желтая точка экстремума — только если анимация дошла до конца и ничего не выбрано в таблице
                    if !self.is_animating
                        && self.selected_iteration.is_none()
                        && self.current_step == res.history.len()
                    {
                        plot_ui.points(
                            Points::new(vec![[res.x_opt, res.f_opt]])
                                .color(egui::Color32::YELLOW)
                                .radius(8.0)
                                .name("Экстремум"),
                        );
                    }
                }
            });
        });
    }

    pub fn render_export_dialog(&mut self, ctx: &egui::Context) {
        if self.show_export_dialog {
            egui::Window::new("📤 Настройки экспорта DOCX")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_width(400.0);
                    ui.spacing_mut().item_spacing.y = 15.0;
                    ui.label(
                        egui::RichText::new("Выберите формат отчета:")
                            .size(16.0)
                            .strong(),
                    );

                    ui.group(|ui| {
                        ui.radio_value(&mut self.export_mode, 0, "🎯 Только финальный результат");
                        ui.add_enabled_ui(self.export_mode == 0, |ui| {
                            ui.indent("f_i", |ui| {
                                ui.label(
                                    egui::RichText::new(
                                        "В отчет попадет итоговый график и полная таблица.",
                                    )
                                    .weak(),
                                );
                            });
                        });
                    });

                    ui.group(|ui| {
                        ui.radio_value(&mut self.export_mode, 1, "🎞 Пошаговый отчет");
                        ui.add_enabled_ui(self.export_mode == 1, |ui| {
                            ui.indent("s_s", |ui| {
                                egui::Grid::new("ex_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 10.0])
                                    .show(ui, |ui| {
                                        ui.label("Начать с:");
                                        ui.add(egui::Slider::new(
                                            &mut self.export_start_step,
                                            1..=self.export_end_step,
                                        ));
                                        ui.end_row();
                                        ui.label("Конец:");
                                        let max = self
                                            .result
                                            .as_ref()
                                            .map(|r| r.history.len())
                                            .unwrap_or(1);
                                        ui.add(egui::Slider::new(
                                            &mut self.export_end_step,
                                            self.export_start_step..=max,
                                        ));
                                        ui.end_row();
                                        ui.label("Интервал:");
                                        ui.add(
                                            egui::Slider::new(
                                                &mut self.export_step_interval,
                                                1..=10,
                                            )
                                            .suffix(" шаг"),
                                        );
                                        ui.end_row();
                                    });
                                ui.checkbox(
                                    &mut self.export_include_table,
                                    "Включить итоговую таблицу",
                                );
                            });
                        });
                    });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([100.0, 30.0], egui::Button::new("❌ Отмена"))
                            .clicked()
                        {
                            self.show_export_dialog = false;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let text = if self.export_mode == 0 {
                                "✅ Создать краткий"
                            } else {
                                "✅ Создать пошаговый"
                            };
                            if ui
                                .add_sized(
                                    [200.0, 30.0],
                                    egui::Button::new(egui::RichText::new(text).strong())
                                        .fill(egui::Color32::from_rgb(40, 80, 40)),
                                )
                                .clicked()
                            {
                                self.save_docx_report();
                                self.show_export_dialog = false;
                            }
                        });
                    });
                });
        }
    }
}
