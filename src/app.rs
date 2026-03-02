use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints, Points, VLine};
use std::cell::Cell;
use std::fs::File;
use std::io::Write;

use crate::logic::{self, Func};
use crate::models::OptimizationResult;

pub struct OptimizationApp {
    // Входные параметры
    a: f64,
    b: f64,
    eps: f64,
    l: f64,

    // Состояние выбора
    selected_func: usize,   // 0: F1, 1: F2
    selected_method: usize, // 0: Дихотомия, 1: Золотое сечение, 2: Фибоначчи

    // Результаты
    result: Option<OptimizationResult>,
    error_msg: Option<String>,
    needs_plot_reset: bool,
}

impl Default for OptimizationApp {
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
        }
    }
}

impl OptimizationApp {
    // Математическое определение функций
    fn get_f_value(&self, x: f64) -> f64 {
        match self.selected_func {
            0 => 3.0 * x - x.powi(3),                             // F1: 3x - x^3
            1 => (9.0 - x.powi(2)) / (x.powi(2) + 2.0 * x + 3.0), // F2: (9-x^2)/(x^2+2x+3)
            _ => 0.0,
        }
    }

    // Запуск расчетов
    fn run_optimization(&mut self) {
        self.error_msg = None;

        // Оборачиваем функцию для подсчета вызовов
        let f_raw = |x: f64| self.get_f_value(x);
        let f_wrapper = Func {
            f: &f_raw,
            calls: Cell::new(0),
        };

        // Определяем, ищем ли мы максимум или минимум
        // По заданию: F1 -> Max, F2 -> Min
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

    // Сохранение отчета в .txt
    fn save_report(&self) {
        if let Some(res) = &self.result {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Text File", &["txt"][..])
                .set_file_name("report.txt")
                .save_file()
            {
                let mut file = File::create(path).unwrap();
                writeln!(file, "ОТЧЕТ ПО ОПТИМИЗАЦИИ").unwrap();
                writeln!(
                    file,
                    "Метод: {}",
                    match self.selected_method {
                        0 => "Дихотомия",
                        1 => "Золотое сечение",
                        _ => "Фибоначчи",
                    }
                )
                .unwrap();
                writeln!(
                    file,
                    "Параметры: a={:.4}, b={:.4}, eps={:.4}, l={:.4}",
                    self.a, self.b, self.eps, self.l
                )
                .unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
                writeln!(file, "x* = {:.6}", res.x_opt).unwrap();
                writeln!(file, "f(x*) = {:.6}", res.f_opt).unwrap();
                writeln!(file, "Вызовов функции: {}", res.fn_calls).unwrap();
                writeln!(file, "--------------------------------------------------").unwrap();
                writeln!(file, "K\ta_k\tb_k\tlambda\tmu\tF(l)\tF(m)").unwrap();
                for it in &res.history {
                    writeln!(
                        file,
                        "{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.4}",
                        it.k, it.a, it.b, it.lambda, it.mu, it.f_lambda, it.f_mu
                    )
                    .unwrap();
                }
            }
        }
    }
}
impl eframe::App for OptimizationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. ЛЕВАЯ ПАНЕЛЬ (Фиксированная ширина)
        egui::SidePanel::left("controls")
            .resizable(false) // Запрещаем менять размер, чтобы оставался одного размера
            .exact_width(260.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("⚙ Настройки");
                    ui.separator();

                    ui.label("Выбор функции:");
                    ui.radio_value(&mut self.selected_func, 0, "F1: 3x - x³ (Max)");
                    ui.radio_value(&mut self.selected_func, 1, "F2: (9-x²)/(x²+2x+3) (Min)");

                    ui.add_space(10.0);
                    ui.label("Метод оптимизации:");
                    ui.radio_value(&mut self.selected_method, 0, "Дихотомия");
                    ui.radio_value(&mut self.selected_method, 1, "Золотое сечение");
                    ui.radio_value(&mut self.selected_method, 2, "Фибоначчи");

                    ui.add_space(10.0);
                    ui.label("Интервал поиска [a, b]:");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.a).speed(0.1));
                        ui.add(egui::DragValue::new(&mut self.b).speed(0.1));
                    });

                    ui.add_space(5.0);
                    ui.label("Параметры ε и l:");
                    egui::Grid::new("params_grid").show(ui, |ui| {
                        ui.label("ε:");
                        ui.add(egui::DragValue::new(&mut self.eps).speed(0.001));
                        ui.end_row();
                        ui.label("l:");
                        ui.add(egui::DragValue::new(&mut self.l).speed(0.001));
                        ui.end_row();
                    });

                    ui.add_space(20.0);
                    if ui
                        .add(egui::Button::new("🚀 РАССЧИТАТЬ").min_size(egui::vec2(240.0, 40.0)))
                        .clicked()
                    {
                        self.run_optimization();
                        self.needs_plot_reset = true; // Сбрасываем камеру на результат
                    }

                    if let Some(err) = &self.error_msg {
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::RED, err);
                    }

                    if self.result.is_some() {
                        ui.add_space(10.0);
                        if ui
                            .add(
                                egui::Button::new("💾 Сохранить отчет в .txt")
                                    .min_size(egui::vec2(240.0, 30.0)),
                            )
                            .clicked()
                        {
                            self.save_report();
                        }
                    }

                    if let Some(res) = &self.result {
                        ui.add_space(20.0);
                        ui.separator();
                        ui.heading("🏁 Результат:");
                        ui.label(format!("x* = {:.6}", res.x_opt));
                        ui.label(format!("f(x*) = {:.6}", res.f_opt));
                        ui.label(format!("Вызовов f: {}", res.fn_calls));
                    }
                });
            });

        // 2. НИЖНЯЯ ПАНЕЛЬ (Фиксированная высота для таблицы)
        egui::TopBottomPanel::bottom("table_panel")
            .resizable(false) // Высота будет фиксированной
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.heading("📊 Таблица итераций");
                ui.separator();

                if let Some(res) = &self.result {
                    let available_width = ui.available_width() - 20.0;
                    let col_width = available_width / 7.0;

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Grid::new("it_grid")
                                .striped(true)
                                .min_col_width(col_width)
                                .max_col_width(col_width)
                                .show(ui, |ui| {
                                    ui.vertical_centered(|ui| ui.strong("K"));
                                    ui.vertical_centered(|ui| ui.strong("a_k"));
                                    ui.vertical_centered(|ui| ui.strong("b_k"));
                                    ui.vertical_centered(|ui| ui.strong("λ"));
                                    ui.vertical_centered(|ui| ui.strong("μ"));
                                    ui.vertical_centered(|ui| ui.strong("F(λ)"));
                                    ui.vertical_centered(|ui| ui.strong("F(μ)"));
                                    ui.end_row();

                                    for it in &res.history {
                                        ui.vertical_centered(|ui| ui.label(it.k.to_string()));
                                        ui.vertical_centered(|ui| ui.label(format!("{:.4}", it.a)));
                                        ui.vertical_centered(|ui| ui.label(format!("{:.4}", it.b)));
                                        ui.vertical_centered(|ui| {
                                            ui.label(format!("{:.4}", it.lambda))
                                        });
                                        ui.vertical_centered(|ui| {
                                            ui.label(format!("{:.4}", it.mu))
                                        });
                                        ui.vertical_centered(|ui| {
                                            ui.label(format!("{:.4}", it.f_lambda))
                                        });
                                        ui.vertical_centered(|ui| {
                                            ui.label(format!("{:.4}", it.f_mu))
                                        });
                                        ui.end_row();
                                    }
                                });
                        });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Данные отсутствуют. Задайте параметры и нажмите 'Рассчитать'.");
                    });
                }
            });

        // 3. ЦЕНТРАЛЬНАЯ ПАНЕЛЬ (График - занимает всё оставшееся место)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📈 График функции и интервалы");

            let plot = Plot::new("Optimization Plot")
                .legend(Legend::default())
                .show_axes([true, true])
                .data_aspect(1.0)
                .allow_zoom(true)
                .allow_drag(true);

            plot.show(ui, |plot_ui| {
                // УСТАНОВКА ГРАНИЦ ТОЛЬКО ПО ТРИГГЕРУ
                if self.needs_plot_reset {
                    let width = (self.b - self.a).abs();
                    let margin = if width < 1e-5 { 1.0 } else { width * 0.5 };

                    plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                        [(self.a - margin) as f64, -5.0],
                        [(self.b + margin) as f64, 5.0],
                    ));

                    // После установки сбрасываем флаг, чтобы не мешать пользователю
                    self.needs_plot_reset = false;
                }

                // --- РИСОВАНИЕ ФУНКЦИИ ---
                // Используем большой span, чтобы при зуме/драге линия не кончалась слишком быстро
                let span = 50.0;
                let center_x = ((self.a + self.b) / 2.0) as f64;
                let n = 2000;
                let points: PlotPoints = (0..=n)
                    .map(|i| {
                        let x = (center_x - span) + (i as f64 / n as f64) * (span * 2.0);
                        [x, self.get_f_value(x)]
                    })
                    .collect();

                plot_ui.line(
                    Line::new(points)
                        .name("f(x)")
                        .color(egui::Color32::LIGHT_BLUE),
                );

                // 3. Рисуем СТАРТОВЫЙ интервал (Синие линии)
                // Они будут внутри отрисованной кривой, так как кривая шире
                plot_ui.vline(
                    VLine::new(self.a as f64)
                        .color(egui::Color32::from_rgb(50, 50, 255))
                        .name("Старт a")
                        .style(egui_plot::LineStyle::Solid),
                );
                plot_ui.vline(
                    VLine::new(self.b as f64)
                        .color(egui::Color32::from_rgb(50, 50, 255))
                        .name("Старт b")
                        .style(egui_plot::LineStyle::Solid),
                );

                // 4. Отрисовка результатов (если расчет произведен)
                if let Some(res) = &self.result {
                    if let Some(last_it) = res.history.last() {
                        // Конечный интервал неопределенности (Красные линии)
                        plot_ui.vline(
                            VLine::new(last_it.a as f64)
                                .color(egui::Color32::RED)
                                .name("Итог a"),
                        );
                        plot_ui.vline(
                            VLine::new(last_it.b as f64)
                                .color(egui::Color32::RED)
                                .name("Итог b"),
                        );
                    }

                    // Точка найденного экстремума (Желтый круг)
                    plot_ui.points(
                        Points::new(vec![[res.x_opt as f64, res.f_opt as f64]])
                            .color(egui::Color32::YELLOW)
                            .radius(6.0)
                            .name("Экстремум"),
                    );
                }
            });
        });
    }
}
