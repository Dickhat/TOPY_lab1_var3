use docx_rs::*;
use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints, Points, VLine};
use image::ImageEncoder;
use plotters::style::full_palette::ORANGE;
use std::cell::Cell;
use std::fs::File;
use std::io::Write;

use crate::logic::{self, Func};
use crate::models::OptimizationResult;

fn lerp(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t.clamp(0.0, 1.0)
}

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
    selected_iteration: Option<usize>, // Для выделения итерации на графике

    // Поля для анимации
    is_animating: bool,
    current_step: usize,  // Сколько шагов сейчас отображать
    last_step_time: f64,  // Время последнего переключения шага
    animation_speed: f32, // Секунд на один шаг
    is_smooth_enabled: bool,
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
            selected_iteration: None,
            animation_speed: 0.5, // По умолчанию полсекунды на шаг
            is_animating: false,
            current_step: 0,
            last_step_time: 0.0,
            is_smooth_enabled: true,
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
        self.selected_iteration = None; // Сбрасываем выделение итерации при новом расчете

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

    // Вспомогательная функция для создания PNG-байтов графика
    fn generate_plot_png(&self) -> Vec<u8> {
        use plotters::prelude::*;

        let mut buffer = vec![0; 800 * 600 * 3]; // Буфер для картинки 800x600
        let res = self.result.as_ref().unwrap();

        {
            let root = BitMapBackend::with_buffer(&mut buffer, (800, 600)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            // Определяем границы (те же 20% отступа)
            let width = (self.b - self.a).abs();
            let margin = if width < 1e-5 { 1.0 } else { width * 0.2 };
            let x_range = (self.a - margin)..(self.b + margin);

            // Находим min/max по Y для красоты
            let y_min = -5.0; // Можно сделать расчет динамическим
            let y_max = 5.0;

            let mut chart = ChartBuilder::on(&root)
                .margin(20)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(x_range, y_min..y_max)
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            // 1. Рисуем кривую функции (голубая)
            chart
                .draw_series(LineSeries::new(
                    (0..100).map(|x| {
                        let curr_x =
                            (self.a - margin) + (x as f64 / 100.0) * (width + 2.0 * margin);
                        (curr_x, self.get_f_value(curr_x))
                    }),
                    &BLUE.mix(0.5),
                ))
                .unwrap();

            // 2. Рисуем стартовые границы (синие)
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(self.a, y_min), (self.a, y_max)],
                    &BLUE,
                )))
                .unwrap();
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(self.b, y_min), (self.b, y_max)],
                    &BLUE,
                )))
                .unwrap();

            // 3. Рисуем точки всех итераций (тусклые)
            for it in &res.history {
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (it.lambda, it.f_lambda),
                        2,
                        &ORANGE.mix(0.3),
                    )))
                    .unwrap();
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (it.mu, it.f_mu),
                        2,
                        &ORANGE.mix(0.3),
                    )))
                    .unwrap();
            }

            // 4. ФИНАЛЬНАЯ ТОЧКА (Ярко-красная и большая)
            chart
                .draw_series(std::iter::once(Circle::new(
                    (res.x_opt, res.f_opt),
                    7,
                    RED.filled(),
                )))
                .unwrap();

            chart
                .draw_series(std::iter::once(Text::new(
                    format!("x*: {:.3}", res.x_opt),
                    (res.x_opt, res.f_opt + 0.3),
                    ("sans-serif", 15).into_font().color(&RED),
                )))
                .unwrap();

            root.present().unwrap();
        }

        // Кодируем сырые пиксели в формат PNG
        let mut png_buffer = Vec::new();
        let img_buffer = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(800, 600, buffer)
            .expect("Failed to create ImageBuffer");
        let encoder = image::codecs::png::PngEncoder::new(&mut png_buffer);
        encoder
            .write_image(img_buffer.as_raw(), 800, 600, image::ColorType::Rgb8)
            .unwrap();

        png_buffer
    }

    fn save_docx_report(&self) {
        if let Some(res) = &self.result {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Word Document", &["docx"][..])
                .set_file_name("optimization_report.docx")
                .save_file()
            {
                // 1. Создаем документ
                let mut doc = Docx::new();

                // 2. Добавляем заголовок
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Отчет по оптимизации").size(32).bold())
                        .align(AlignmentType::Center),
                );

                // --- ДОБАВЛЕНИЕ ГРАФИКА ---
                let png_bytes = self.generate_plot_png();
                let pic = Pic::new(&png_bytes); // Создаем объект картинки для docx

                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_image(pic)) // Вставляем картинку в новый абзац
                        .align(AlignmentType::Center),
                );

                // 3. Общая информация
                let method_name = match self.selected_method {
                    0 => "Дихотомия",
                    1 => "Золотое сечение",
                    _ => "Фибоначчи",
                };

                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text(format!("Метод: {}", method_name))),
                );
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!(
                    "Результат: x* = {:.6}, f(x*) = {:.6}",
                    res.x_opt, res.f_opt
                ))));
                doc = doc.add_paragraph(Paragraph::new().add_run(
                    Run::new().add_text(format!("Количество вычислений: {}", res.fn_calls)),
                ));

                // 4. Создаем ТАБЛИЦУ
                let mut table = Table::new(vec![]);

                // Шапка таблицы
                let header_row = TableRow::new(vec![
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("K").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("a_k").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("b_k").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("λ").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("μ").bold())),
                    TableCell::new().add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text("F(λ)").bold()),
                    ),
                    TableCell::new().add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text("F(μ)").bold()),
                    ),
                ]);
                table = table.add_row(header_row);

                // Наполняем данными из истории
                for it in &res.history {
                    let row = TableRow::new(vec![
                        TableCell::new().add_paragraph(
                            Paragraph::new().add_run(Run::new().add_text(it.k.to_string())),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new().add_run(Run::new().add_text(format!("{:.4}", it.a))),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new().add_run(Run::new().add_text(format!("{:.4}", it.b))),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new()
                                .add_run(Run::new().add_text(format!("{:.4}", it.lambda))),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new().add_run(Run::new().add_text(format!("{:.4}", it.mu))),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new()
                                .add_run(Run::new().add_text(format!("{:.4}", it.f_lambda))),
                        ),
                        TableCell::new().add_paragraph(
                            Paragraph::new()
                                .add_run(Run::new().add_text(format!("{:.4}", it.f_mu))),
                        ),
                    ]);
                    table = table.add_row(row);
                }

                doc = doc.add_table(table);

                // 5. Сохраняем файл
                let file = File::create(path).unwrap();
                doc.build().pack(file).expect("Не удалось создать DOCX");
            }
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
        let mut animation_t = 0.0; // Прогресс текущего шага
        ctx.set_pixels_per_point(1.1);

        // ЛОГИКА АНИМАЦИИ
        if self.is_animating {
            if let Some(res) = &self.result {
                let elapsed = ctx.input(|i| i.time) - self.last_step_time;
                animation_t = (elapsed / self.animation_speed as f64).min(1.0);

                if animation_t >= 1.0 {
                    if self.current_step < res.history.len() {
                        self.current_step += 1;
                        self.last_step_time = ctx.input(|i| i.time);
                        animation_t = 0.0;
                        self.selected_iteration = Some(self.current_step - 1);
                    } else {
                        self.is_animating = false;
                    }
                }
                ctx.request_repaint(); // Важно для плавности!
            }
        }

        // 1. ЛЕВАЯ ПАНЕЛЬ (Фиксированная ширина)
        egui::SidePanel::left("controls")
            .resizable(false)
            .exact_width(280.0) // Немного расширим для удобства
            .show(ctx, |ui| {
                // Добавим внутренние отступы всей панели
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.add_space(10.0);

                ui.vertical_centered(|ui| {
                    ui.heading("📊 Оптимизация");
                });
                ui.separator();

                // БЛОК 1: ВЫБОР ФУНКЦИИ
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎯 Функция").strong());
                    ui.radio_value(&mut self.selected_func, 0, "F1: 3x - x³ (Max)");
                    ui.radio_value(&mut self.selected_func, 1, "F2: (9-x²)/(x²+2x+3) (Min)");
                });

                // БЛОК 2: МЕТОД
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🛠 Метод").strong());
                    egui::ComboBox::from_label("")
                        .selected_text(match self.selected_method {
                            0 => "Дихотомия",
                            1 => "Золотое сечение",
                            _ => "Фибоначчи",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_method, 0, "Дихотомия");
                            ui.selectable_value(&mut self.selected_method, 1, "Золотое сечение");
                            ui.selectable_value(&mut self.selected_method, 2, "Фибоначчи");
                        });
                });

                // БЛОК 3: ПАРАМЕТРЫ (Сетка для красоты)
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
                            }
                            ui.end_row();

                            ui.label("Конец b:");
                            if ui
                                .add(egui::DragValue::new(&mut self.b).speed(0.1))
                                .changed()
                            {
                                self.needs_plot_reset = true;
                            }
                            ui.end_row();

                            ui.label("Точность ε:");
                            ui.add(egui::DragValue::new(&mut self.eps).speed(0.001));
                            ui.end_row();

                            ui.label("Длина l:");
                            ui.add(egui::DragValue::new(&mut self.l).speed(0.001));
                            ui.end_row();
                        });
                });

                // КНОПКА РАССЧИТАТЬ (Уменьшенная и аккуратная)
                ui.vertical_centered(|ui| {
                    let btn = egui::Button::new(egui::RichText::new("🚀 Рассчитать").size(16.0))
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

                // БЛОК 4: АНИМАЦИЯ
                ui.group(|ui| {
                    ui.label(egui::RichText::new("🎬 Анимация").strong());

                    // ГАЛОЧКА ДЛЯ ПЛАВНОСТИ
                    ui.checkbox(&mut self.is_smooth_enabled, "Плавное перемещение");

                    // Настройка скорости
                    ui.add(egui::Slider::new(&mut self.animation_speed, 0.1..=3.0).text("сек/шаг"));

                    // Кнопки управления
                    ui.horizontal(|ui| {
                        if ui.button("▶ Старт").clicked() {
                            if self.result.is_none() {
                                self.run_optimization();
                            }
                            self.current_step = 0;
                            self.is_animating = true;
                            self.last_step_time = ctx.input(|i| i.time);
                        }
                        if ui.button("⏸ Стоп").clicked() {
                            self.is_animating = false;
                        }
                        if ui.button("🔄 Сброс").clicked() {
                            self.is_animating = false;
                            self.current_step = 0;
                            self.selected_iteration = None;
                        }
                    });

                    // Твой возвращенный код слайдера шагов
                    if let Some(res) = &self.result {
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);

                        // Слайдер ручного прокручивания шагов
                        if ui
                            .add(
                                egui::Slider::new(&mut self.current_step, 0..=res.history.len())
                                    .text("Тек. шаг"),
                            )
                            .changed()
                        {
                            self.is_animating = false; // При ручном сдвиге выключаем авто-анимацию
                            if self.current_step > 0 {
                                self.selected_iteration = Some(self.current_step - 1);
                            } else {
                                self.selected_iteration = None;
                            }
                        }
                    }
                });

                // ЭКСПОРТ
                if self.result.is_some() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Экспорт:");
                        if ui.button("📄 TXT").clicked() {
                            self.save_report();
                        }
                        if ui.button("📝 DOCX").clicked() {
                            self.save_docx_report();
                        }
                    });
                }

                // РЕЗУЛЬТАТЫ (Красивая карточка)
                if let Some(res) = &self.result {
                    ui.add_space(5.0);
                    egui::Frame::none()
                        .fill(ui.visuals().faint_bg_color)
                        .rounding(5.0)
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.heading("🏁 Итог");
                            ui.label(format!("x*: {:.6}", res.x_opt));
                            ui.label(format!("f(x*): {:.6}", res.f_opt));
                            ui.label(format!("Шагов: {}", res.history.len()));
                        });
                }
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

                                    for (i, it) in
                                        res.history.iter().take(self.current_step).enumerate()
                                    {
                                        let is_selected = self.selected_iteration == Some(i);

                                        // Отрисовываем первую колонку (K) как переключатель
                                        if ui
                                            .selectable_label(is_selected, it.k.to_string())
                                            .clicked()
                                        {
                                            if is_selected {
                                                self.selected_iteration = None; // Снять выделение при повторном клике
                                            } else {
                                                self.selected_iteration = Some(i);
                                            }
                                        }

                                        // Остальные колонки просто отображаем (можно тоже сделать кликабельными при желании)
                                        ui.label(format!("{:.4}", it.a));
                                        ui.label(format!("{:.4}", it.b));
                                        ui.label(format!("{:.4}", it.lambda));
                                        ui.label(format!("{:.4}", it.mu));
                                        ui.label(format!("{:.4}", it.f_lambda));
                                        ui.label(format!("{:.4}", it.f_mu));
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

                if let Some(res) = &self.result {
                    // РЕШАЕМ: использовать реальное время для интерполяции или "заморозить" его
                    // Если плавность выключена, drawing_t всегда 0.0, и lerp возвращает 'start'
                    let drawing_t = if self.is_smooth_enabled {
                        animation_t
                    } else {
                        0.0
                    };
                    // Вычисляем текущие значения границ
                    let (cur_a, cur_b, cur_lam, cur_mu);

                    if self.current_step == 0 {
                        // Анимируем от начального интервала к первой итерации
                        let next = &res.history[0];
                        cur_a = lerp(self.a, next.a, drawing_t);
                        cur_b = lerp(self.b, next.b, drawing_t);
                        cur_lam = lerp(self.a, next.lambda, drawing_t);
                        cur_mu = lerp(self.b, next.mu, drawing_t);
                    } else if self.current_step < res.history.len() {
                        // Анимируем между итерациями k и k+1
                        let prev = &res.history[self.current_step - 1];
                        let next = &res.history[self.current_step];
                        cur_a = lerp(prev.a, next.a, drawing_t);
                        cur_b = lerp(prev.b, next.b, drawing_t);
                        cur_lam = lerp(prev.lambda, next.lambda, drawing_t);
                        cur_mu = lerp(prev.mu, next.mu, drawing_t);
                    } else {
                        // Финальное состояние
                        let last = res.history.last().unwrap();
                        cur_a = last.a;
                        cur_b = last.b;
                        cur_lam = last.lambda;
                        cur_mu = last.mu;
                    }

                    // РИСУЕМ ПЛАВНЫЕ ЛИНИИ
                    let orange = egui::Color32::from_rgb(255, 165, 0);
                    plot_ui.vline(VLine::new(cur_a).color(orange).width(2.0));
                    plot_ui.vline(VLine::new(cur_b).color(orange).width(2.0));

                    // Плавные точки lambda/mu
                    plot_ui.points(
                        Points::new(vec![
                            [cur_lam, self.get_f_value(cur_lam)],
                            [cur_mu, self.get_f_value(cur_mu)],
                        ])
                        .color(orange)
                        .radius(4.0),
                    );

                    // Когда закончили - рисуем желтую точку
                    if !self.is_animating && self.current_step == res.history.len() {
                        plot_ui.points(
                            Points::new(vec![[res.x_opt, res.f_opt]])
                                .color(egui::Color32::YELLOW)
                                .radius(8.0),
                        );
                    }
                }
            });
        });
    }
}
