use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points, Corner, Legend};
use std::f64::consts::PI;

// Данные для строки таблицы
#[derive(Clone, Debug)]
struct TableRow {
    k: String,
    delta: String,
    xk: String,
    fxk: String,
    j: String,
    yj: String,
    fyj: String,
    dj: String,
    y_plus: String,
    f_plus: String,
    y_minus: String,
    f_minus: String,
    description: String,
}

// Основные данные итерации
#[derive(Clone, Debug)]
struct IterationData {
    k: usize,
    delta: f64,
    xk: Vec<f64>,
    fxk: f64,
    steps: Vec<ExploratoryStep>,
    pattern_info: Option<PatternInfo>,
}

#[derive(Clone, Debug)]
struct ExploratoryStep {
    j: usize,
    yj: Vec<f64>,
    fyj: f64,
    dj: Vec<f64>,
    y_plus: Option<(Vec<f64>, f64)>,
    y_minus: Option<(Vec<f64>, f64)>,
    description: String,
}

#[derive(Clone, Debug)]
struct PatternInfo {
    success: bool,
    description: String,
}

struct HookeJeevesApp {
    function_choice: usize,
    initial_point_2d: [f64; 2],
    initial_point_3d: [f64; 3],
    epsilon: f64,
    initial_delta: f64,
    alpha: f64,
    iterations: Vec<IterationData>,
    table_rows: Vec<TableRow>,
    optimal_point: Vec<f64>,
    optimal_value: f64,
    total_iterations: usize,
    computation_done: bool,
    status_message: String,
    show_plot: bool,
}

impl HookeJeevesApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            function_choice: 0,
            initial_point_2d: [1.0, 1.0],
            initial_point_3d: [1.0, 1.0, 1.0],
            epsilon: 0.001,
            initial_delta: 0.2,
            alpha: 1.0,
            iterations: Vec::new(),
            table_rows: Vec::new(),
            optimal_point: Vec::new(),
            optimal_value: 0.0,
            total_iterations: 0,
            computation_done: false,
            status_message: String::new(),
            show_plot: true,
        }
    }

    fn f1(x: &[f64]) -> f64 {
        -6.0 * x[0] - 4.0 * x[1] + x[0].powi(2) + x[1].powi(2) + 18.0
    }

    fn f2(x: &[f64]) -> f64 {
        4.0 * x[0].powi(2) + 3.0 * x[1].powi(2) + x[2].powi(2) 
        + 4.0 * x[0] * x[1] - 2.0 * x[1] * x[2] - 16.0 * x[0] - 4.0 * x[2]
    }

    fn evaluate_function(&self, x: &[f64]) -> f64 {
        match self.function_choice {
            0 => Self::f1(x),
            1 => Self::f2(x),
            _ => 0.0,
        }
    }

    fn get_dimension(&self) -> usize {
        match self.function_choice {
            0 => 2,
            1 => 3,
            _ => 2,
        }
    }

    fn get_initial_point(&self) -> Vec<f64> {
        match self.function_choice {
            0 => self.initial_point_2d.to_vec(),
            1 => self.initial_point_3d.to_vec(),
            _ => vec![],
        }
    }

    fn fmt_vec(v: &[f64]) -> String {
        format!("({})", v.iter().map(|x| format!("{:.2}", x)).collect::<Vec<_>>().join(", "))
    }

    fn optimize(&mut self) {
        self.iterations.clear();
        self.table_rows.clear();
        
        let mut x = self.get_initial_point();
        let mut delta = self.initial_delta;
        let n = self.get_dimension();
        let mut k = 1;
        let mut iter_count = 0;

        while delta > self.epsilon {
            iter_count += 1;
            let xk = x.clone();
            let fxk = self.evaluate_function(&xk);
            let mut y = xk.clone();
            let mut steps: Vec<ExploratoryStep> = Vec::new();

            // Исследующий поиск
            for j in 0..n {
                let mut dj = vec![0.0; n];
                dj[j] = 1.0;

                let fyj = self.evaluate_function(&y);
                let mut y_plus = y.clone();
                y_plus[j] += delta;
                let f_plus = self.evaluate_function(&y_plus);

                let mut y_minus = y.clone();
                y_minus[j] -= delta;
                let f_minus = self.evaluate_function(&y_minus);

                let mut desc = String::new();
                if f_plus < fyj {
                    y = y_plus.clone();
                    desc = "Шаг вперёд успешен: F(y+Δd) < F(y). Точка обновлена.".to_string();
                } else if f_minus < fyj {
                    y = y_minus.clone();
                    desc = "Шаг назад успешен: F(y-Δd) < F(y). Точка обновлена.".to_string();
                } else {
                    desc = "Шаг неуспешен: нет улучшения ни вперёд, ни назад. Точка сохранена.".to_string();
                }

                steps.push(ExploratoryStep {
                    j: j + 1,
                    yj: if j == 0 { xk.clone() } else { steps.last().unwrap().yj.clone() },
                    fyj: if j == 0 { fxk } else { steps.last().map(|s| s.fyj).unwrap_or(fxk) },
                    dj,
                    y_plus: Some((y_plus.clone(), f_plus)),
                    y_minus: Some((y_minus.clone(), f_minus)),
                    description: desc.to_string(),
                });
            }

            // Корректировка yj для точности таблицы
            let mut cur_y = xk.clone();
            for (idx, step) in steps.iter_mut().enumerate() {
                step.yj = cur_y.clone();
                step.fyj = self.evaluate_function(&cur_y);
                
                let mut dj = vec![0.0; n];
                dj[idx] = 1.0;
                step.dj = dj;

                let mut y_p = cur_y.clone();
                y_p[idx] += delta;
                let f_p = self.evaluate_function(&y_p);
                step.y_plus = Some((y_p.clone(), f_p));

                let mut y_m = cur_y.clone();
                y_m[idx] -= delta;
                let f_m = self.evaluate_function(&y_m);
                step.y_minus = Some((y_m.clone(), f_m));

                if f_p < step.fyj {
                    cur_y = y_p;
                    step.description = "Шаг вперёд успешен: F(y+Δd) < F(y). Переход к y+Δd.".to_string();
                } else if f_m < step.fyj {
                    cur_y = y_m;
                    step.description = "Шаг назад успешен: F(y-Δd) < F(y). Переход к y-Δd.".to_string();
                } else {
                    step.description = "Шаг неуспешен: F(y±Δd) ≥ F(y). Точка не меняется.".to_string();
                }
            }

            // Проверка результата исследующего поиска
            let fy_next = self.evaluate_function(&cur_y);
            let mut pattern_info = None;
            
            if fy_next < fxk {
                // Ускоряющий шаг
                let dir: Vec<f64> = cur_y.iter().zip(xk.iter()).map(|(&a, &b)| a - b).collect();
                let mut x_new = cur_y.iter().zip(dir.iter()).map(|(&a, &b)| a + self.alpha * b).collect::<Vec<_>>();
                let f_new = self.evaluate_function(&x_new);
                
                if f_new < fy_next {
                    x = x_new;
                    pattern_info = Some(PatternInfo { success: true, description: "Ускоряющий шаг успешен: F(x_new) < F(y). Применяем x_new.".to_string() });
                } else {
                    x = cur_y;
                    pattern_info = Some(PatternInfo { success: false, description: "Ускоряющий шаг не улучшил: F(x_new) ≥ F(y). Принимаем точку исследующего поиска y.".to_string() });
                }
            } else {
                delta /= 2.0;
                x = cur_y; // или оставляем xk, по учебнику обычно возвращаемся к базе
                pattern_info = Some(PatternInfo { success: false, description: "Исследующий поиск не дал улучшения. Шаг Δ уменьшен вдвое.".to_string() });
            }

            self.iterations.push(IterationData {
                k,
                delta,
                xk: xk.clone(),
                fxk,
                steps,
                pattern_info,
            });

            // Формирование строк таблицы
            for (i, step) in self.iterations.last().unwrap().steps.iter().enumerate() {
                self.table_rows.push(TableRow {
                    k: if i == 0 { k.to_string() } else { "".to_string() },
                    delta: if i == 0 { format!("{:.2}", self.iterations.last().unwrap().delta) } else { "".to_string() },
                    xk: if i == 0 { Self::fmt_vec(&xk) } else { "".to_string() },
                    fxk: if i == 0 { format!("{:.2}", fxk) } else { "".to_string() },
                    j: step.j.to_string(),
                    yj: Self::fmt_vec(&step.yj),
                    fyj: format!("{:.2}", step.fyj),
                    dj: Self::fmt_vec(&step.dj),
                    y_plus: step.y_plus.as_ref().map(|(p, _)| Self::fmt_vec(p)).unwrap_or("-".to_string()),
                    f_plus: step.y_plus.as_ref().map(|(_, v)| format!("{:.2}", v)).unwrap_or("-".to_string()),
                    y_minus: step.y_minus.as_ref().map(|(p, _)| Self::fmt_vec(p)).unwrap_or("-".to_string()),
                    f_minus: step.y_minus.as_ref().map(|(_, v)| format!("{:.2}", v)).unwrap_or("-".to_string()),
                    description: step.description.clone(),
                });
            }
            if let Some(pi) = &self.iterations.last().unwrap().pattern_info {
                self.table_rows.push(TableRow {
                    k: "".to_string(), delta: "".to_string(), xk: "".to_string(), fxk: "".to_string(),
                    j: "-".to_string(), yj: "-".to_string(), fyj: "-".to_string(), dj: "-".to_string(),
                    y_plus: "-".to_string(), f_plus: "-".to_string(), y_minus: "-".to_string(), f_minus: "-".to_string(),
                    description: pi.description.clone(),
                });
            }

            k += 1;
            if iter_count > 1000 {
                self.status_message = "Предупреждение: достигнуто максимальное число итераций".to_string();
                break;
            }
        }

        if let Some(last) = self.iterations.last() {
            self.optimal_point = last.xk.clone();
            self.optimal_value = last.fxk;
        }
        self.total_iterations = iter_count;
        self.computation_done = true;
        self.status_message = "Оптимизация успешно завершена!".to_string();
    }

    fn generate_level_lines_data(&self) -> (Vec<PlotPoints>, Vec<[f64; 2]>) {
        let mut level_lines = Vec::new();
        let mut path_points = Vec::new();

        if self.function_choice != 0 {
            return (level_lines, path_points);
        }

        for iter in &self.iterations {
            if iter.xk.len() >= 2 {
                path_points.push([iter.xk[0], iter.xk[1]]);
            }
        }

        let levels: Vec<f64> = vec![8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 25.0, 30.0];
        for level in levels {
            let radius_sq = level - 5.0;
            if radius_sq > 0.0 {
                let radius = radius_sq.sqrt();
                let mut points = Vec::new();
                for i in 0..=100 {
                    let angle = (i as f64) / 100.0 * 2.0 * PI;
                    let x = 3.0 + radius * angle.cos();
                    let y = 2.0 + radius * angle.sin();
                    points.push([x, y]);
                }
                level_lines.push(PlotPoints::from(points));
            }
        }
        (level_lines, path_points)
    }
}

impl eframe::App for HookeJeevesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Файл", |ui| {
                    if ui.button("Сохранить результаты").clicked() {
                        let mut output = String::new();
                        output.push_str("Результаты оптимизации методом Хука-Дживса\n\n");
                        output.push_str(&format!("Функция: F{}\n", self.function_choice + 1));
                        output.push_str(&format!("Оптимальная точка: {:?}\n", self.optimal_point));
                        output.push_str(&format!("Оптимальное значение: {}\n", self.optimal_value));
                        output.push_str(&format!("Всего итераций: {}\n", self.total_iterations));
                        std::fs::write("результаты.txt", output).ok();
                        self.status_message = "Результаты сохранены в файл результаты.txt".to_string();
                    }
                });
            });
        });

        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            ui.heading("Метод Хука и Дживса");
            ui.separator();
            
            ui.label("Выберите функцию:");
            ui.radio_value(&mut self.function_choice, 0, "F₁(x) = -6X₁ - 4X₂ + X₁² + X₂² + 18");
            ui.radio_value(&mut self.function_choice, 1, "F₂(x) = 4X₁² + 3X₂² + X₃² + 4X₁X₂ - 2X₂X₃ - 16X₁ - 4X₃");
            
            ui.separator();
            ui.label("Параметры:");
            
            if self.function_choice == 0 {
                ui.horizontal(|ui| {
                    ui.label("Начальная точка (X₁, X₂):");
                    ui.add(egui::DragValue::new(&mut self.initial_point_2d[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_2d[1]).speed(0.1));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Начальная точка:");
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[1]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[2]).speed(0.1));
                });
            }
            
            ui.horizontal(|ui| {
                ui.label("Точность (ε):");
                ui.add(egui::DragValue::new(&mut self.epsilon).speed(0.001));
            });
            ui.horizontal(|ui| {
                ui.label("Начальный шаг (Δ):");
                ui.add(egui::DragValue::new(&mut self.initial_delta).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Ускорение (α):");
                ui.add(egui::DragValue::new(&mut self.alpha).speed(0.1));
            });
            
            ui.separator();
            if ui.button("Запустить оптимизацию").clicked() {
                self.optimize();
            }
            if ui.button("Тест разных ε").clicked() {
                for &eps in &[0.1, 0.01, 0.001] {
                    self.epsilon = eps;
                    self.optimize();
                }
                self.status_message = "Тест завершён. Проверьте таблицу.".to_string();
            }
            
            ui.separator();
            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.computation_done {
                // Вертикальная прокручиваемая область
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Результаты оптимизации");
                    ui.separator();
                    
                    // График (всегда показывается для F1)
                    if self.function_choice == 0 {
                        ui.heading("Линии уровня и траектория поиска");
                        let (level_lines, path_points) = self.generate_level_lines_data();
                        
                        Plot::new("optimization_plot")
                            .view_aspect(1.0)
                            .width(500.0)
                            .height(400.0)
                            .show_axes([true, true])
                            .show_grid(true)
                            .legend(Legend::default().position(Corner::LeftTop))
                            .label_formatter(|name, value| {
                                if name.is_empty() {
                                    format!("x: {:.2}\ny: {:.2}", value.x, value.y)
                                } else {
                                    format!("{}: x={:.2}, y={:.2}", name, value.x, value.y)
                                }
                            })
                            .show(ui, |plot_ui| {
                                for (i, line_points) in level_lines.into_iter().enumerate() {
                                    plot_ui.line(
                                        Line::new(line_points)
                                            .color(egui::Color32::from_rgb(100, 100, 200))
                                            .width(1.0)
                                            .name(format!("Уровень {}", i))
                                    );
                                }
                                
                                if path_points.len() > 1 {
                                    plot_ui.line(
                                        Line::new(PlotPoints::from(path_points.clone()))
                                            .color(egui::Color32::RED)
                                            .width(2.0)
                                            .name("Траектория")
                                    );
                                    plot_ui.points(
                                        Points::new(PlotPoints::from(path_points.clone()))
                                            .color(egui::Color32::RED)
                                            .radius(3.0)
                                            .name("Точки итераций")
                                    );
                                }
                                
                                if !path_points.is_empty() {
                                    plot_ui.points(
                                        Points::new(PlotPoints::from(vec![path_points[0]]))
                                            .color(egui::Color32::GREEN)
                                            .radius(5.0)
                                            .name("Старт")
                                    );
                                    plot_ui.points(
                                        Points::new(PlotPoints::from(vec![path_points[path_points.len()-1]]))
                                            .color(egui::Color32::BLUE)
                                            .radius(5.0)
                                            .name("Финиш")
                                    );
                                }
                            });
                        
                        ui.separator();
                    }
                    
                    // Таблица итераций
                    ui.heading("Таблица итераций");
                    egui::ScrollArea::horizontal().show(ui, |ui| {
                        egui::Grid::new("iter_table")
                            .num_columns(13)
                            .spacing([4.0, 2.0])
                            .striped(true)
                            .show(ui, |ui| {
                                // Заголовки
                                ui.label("K");
                                ui.label("Δ");
                                ui.label("Xk");
                                ui.label("F(Xk)");
                                ui.label("J");
                                ui.label("Yj");
                                ui.label("F(Yj)");
                                ui.label("di");
                                ui.label("Yj+Δdj");
                                ui.label("F(Yj+Δdj)");
                                ui.label("Yj-Δdj");
                                ui.label("F(Yj-Δdj)");
                                ui.label("Описание");
                                ui.end_row();

                                for row in &self.table_rows {
                                    ui.label(&row.k);
                                    ui.label(&row.delta);
                                    ui.label(&row.xk);
                                    ui.label(&row.fxk);
                                    ui.label(&row.j);
                                    ui.label(&row.yj);
                                    ui.label(&row.fyj);
                                    ui.label(&row.dj);
                                    ui.label(&row.y_plus);
                                    ui.label(&row.f_plus);
                                    ui.label(&row.y_minus);
                                    ui.label(&row.f_minus);
                                    ui.label(&row.description);
                                    ui.end_row();
                                }
                            });
                    });
                    
                    ui.separator();
                    
                    // Финальный результат (выделенный блок)
                    ui.heading("🎯 ФИНАЛЬНЫЙ РЕЗУЛЬТАТ ОПТИМИЗАЦИИ");
                    
                    // Выделяем блок с результатами
                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(230, 240, 255))
                        .inner_margin(egui::Margin::same(15.0))
                        .stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 100, 200)))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(egui::RichText::new("✅ Оптимизация завершена успешно!").size(16.0).strong());
                                ui.separator();
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Функция:").strong());
                                    ui.label(format!("F{}", self.function_choice + 1));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Оптимальная точка:").strong());
                                    ui.label(egui::RichText::new(format!("{:?}", self.optimal_point)).size(14.0).monospace());
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Оптимальное значение:").strong());
                                    ui.label(egui::RichText::new(format!("{:.6}", self.optimal_value)).size(16.0).color(egui::Color32::from_rgb(0, 128, 0)));
                                });
                                
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Количество итераций:").strong());
                                    ui.label(egui::RichText::new(format!("{}", self.total_iterations)).size(14.0));
                                });
                            });
                        });
                    
                    ui.separator();
                });
            } else {
                ui.heading("Оптимизация методом Хука-Дживса с дискретным шагом");
                ui.separator();
                ui.label("Настройте параметры в левой панели и нажмите «Запустить оптимизацию».");
                ui.label("");
                ui.label("Алгоритм выполняет:");
                ui.label("1. Исследующий поиск по координатным направлениям");
                ui.label("2. Ускоряющий шаг при успехе");
                ui.label("3. Уменьшение шага Δ при неудаче");
                ui.label("4. Остановку при Δ < ε");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1500.0, 950.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Оптимизатор Хука-Дживса",
        options,
        Box::new(|cc| Box::new(HookeJeevesApp::new(cc))),
    )
}