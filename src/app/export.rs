use crate::OptimizationApp;
use docx_rs::*;
use std::fs::File;
use std::io::Write;
use image::ImageEncoder;

impl OptimizationApp {
    pub fn generate_plot_png(&self, step_idx: Option<usize>) -> Vec<u8> {
        use plotters::prelude::*;

        let mut buffer = vec![0; 800 * 600 * 3];
        let res = self.result.as_ref().unwrap();

        let (draw_a, draw_b, draw_lam, draw_mu, is_final) = match step_idx {
            Some(idx) => {
                let it = &res.history[idx];
                (it.a, it.b, it.lambda, it.mu, false)
            }
            std::prelude::v1::None => {
                let last = res.history.last().unwrap();
                (last.a, last.b, last.lambda, last.mu, true)
            }
        };

        {
            let root = BitMapBackend::with_buffer(&mut buffer, (800, 600)).into_drawing_area();
            root.fill(&WHITE).unwrap();

            let width = (self.b - self.a).abs();
            let margin = if width < 1e-5 { 1.0 } else { width * 0.2 };
            let x_range = (self.a - margin)..(self.b + margin);

            let mut chart = ChartBuilder::on(&root)
                .margin(30)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(x_range, -6.0..6.0)
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            // 1. Кривая функции
            chart
                .draw_series(LineSeries::new(
                    (0..100).map(|x| {
                        let curr_x =
                            (self.a - margin) + (x as f64 / 100.0) * (width + 2.0 * margin);
                        (curr_x, self.get_f_value(curr_x))
                    }),
                    &BLUE.mix(0.4),
                ))
                .unwrap();

            // 2. Синие границы старта
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(self.a, -6.0), (self.a, 6.0)],
                    &BLUE,
                )))
                .unwrap();
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(self.b, -6.0), (self.b, 6.0)],
                    &BLUE,
                )))
                .unwrap();

            // 3. Облако всех точек поиска (бледное)
            for it in &res.history {
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (it.lambda, it.f_lambda),
                        2,
                        &MAGENTA.mix(0.15),
                    )))
                    .unwrap();
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (it.mu, it.f_mu),
                        2,
                        &MAGENTA.mix(0.15),
                    )))
                    .unwrap();
            }

            // 4. АКЦЕНТ ТЕКУЩЕГО ШАГА (теперь переменные используются)
            let orange_color = &plotters::style::full_palette::ORANGE;
            // Линии границ текущего шага
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(draw_a, -6.0), (draw_a, 6.0)],
                    orange_color,
                )))
                .unwrap();
            chart
                .draw_series(std::iter::once(PathElement::new(
                    vec![(draw_b, -6.0), (draw_b, 6.0)],
                    orange_color,
                )))
                .unwrap();

            // Яркие точки lambda и mu конкретно этого шага
            chart
                .draw_series(std::iter::once(Circle::new(
                    (draw_lam, self.get_f_value(draw_lam)),
                    4,
                    orange_color.filled(),
                )))
                .unwrap();
            chart
                .draw_series(std::iter::once(Circle::new(
                    (draw_mu, self.get_f_value(draw_mu)),
                    4,
                    orange_color.filled(),
                )))
                .unwrap();

            // 5. ФИНАЛЬНАЯ ТОЧКА (если финал)
            if is_final {
                chart
                    .draw_series(std::iter::once(Circle::new(
                        (res.x_opt, res.f_opt),
                        7,
                        RED.filled(),
                    )))
                    .unwrap();
            }

            root.present().unwrap();
        }

        let mut png_buffer = Vec::new();
        let img_buffer =
            image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(800, 600, buffer).unwrap();
        let encoder = image::codecs::png::PngEncoder::new(&mut png_buffer);
        let _ = encoder.write_image(img_buffer.as_raw(), 800, 600, image::ColorType::Rgb8);
        png_buffer
    }
    pub fn save_docx_report(&self) {
        if let Some(res) = &self.result {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Word Document", &["docx"][..])
                .set_file_name("optimization_report.docx")
                .save_file()
            {
                let mut doc = Docx::new();

                // --- ЗАГОЛОВОК ---
                let method_name = match self.selected_method {
                    0 => "Дихотомический поиск",
                    1 => "Метод Золотого сечения",
                    _ => "Метод Фибоначчи",
                };

                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(format!("Отчет по методу: {}", method_name))
                                .size(32)
                                .bold(),
                        )
                        .align(AlignmentType::Center),
                );

                // --- ГРАФИКИ (Пункт 5 задания) ---
                if self.export_mode == 0 {
                    let png_bytes = self.generate_plot_png(None);
                    doc = doc.add_paragraph(
                        Paragraph::new()
                            .add_run(Run::new().add_image(Pic::new(&png_bytes)))
                            .align(AlignmentType::Center),
                    );
                } else {
                    let start_idx = self.export_start_step.saturating_sub(1);
                    let end_idx =
                        (self.export_end_step.saturating_sub(1)).min(res.history.len() - 1);

                    for i in (start_idx..=end_idx).step_by(self.export_step_interval) {
                        let png_bytes = self.generate_plot_png(Some(i));
                        doc =
                            doc.add_paragraph(Paragraph::new().add_run(
                                Run::new().add_text(format!("Итерация №{}", i + 1)).bold(),
                            ));
                        doc = doc.add_paragraph(
                            Paragraph::new()
                                .add_run(Run::new().add_image(Pic::new(&png_bytes)))
                                .align(AlignmentType::Center),
                        );
                    }
                }

                // --- ТАБЛИЦА ИТЕРАЦИЙ (Пункт 3 задания) ---
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_text("Результаты итераций:").bold().size(24)),
                );

                let mut table = Table::new(vec![]);
                // Шапка таблицы (строго по заданию)
                let header = TableRow::new(vec![
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("K").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("a_k").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("b_k").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("λ_k").bold())),
                    TableCell::new()
                        .add_paragraph(Paragraph::new().add_run(Run::new().add_text("μ_k").bold())),
                    TableCell::new().add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text("F(λ_k)").bold()),
                    ),
                    TableCell::new().add_paragraph(
                        Paragraph::new().add_run(Run::new().add_text("F(μ_k)").bold()),
                    ),
                ]);
                table = table.add_row(header);

                // Заполнение данными всех итераций
                for it in &res.history {
                    table = table.add_row(TableRow::new(vec![
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
                    ]));
                }
                doc = doc.add_table(table);

                // --- ИТОГОВАЯ ИНФОРМАЦИЯ (Пункт 4 задания) ---
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(""))); // Отступ
                doc = doc.add_paragraph(
                    Paragraph::new().add_run(
                        Run::new()
                            .add_text("Заключение по вычислениям:")
                            .bold()
                            .size(24),
                    ),
                );

                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!(
                    "• Оптимальное значение аргумента (x*): {:.6}",
                    res.x_opt
                ))));
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!(
                    "• Оптимальное значение функции f(x*): {:.6}",
                    res.f_opt
                ))));
                doc = doc.add_paragraph(Paragraph::new().add_run(Run::new().add_text(format!(
                    "• Общее количество вычислений функции: {}",
                    res.fn_calls
                ))));

                // Сохранение
                let file = File::create(path).unwrap();
                doc.build()
                    .pack(file)
                    .expect("Не удалось создать DOCX файл");
            }
        }
    }
    // Сохранение отчета в .txt
    pub fn save_report(&self) {
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
