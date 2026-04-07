// src/main.rs
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use std::f64::consts::PI;

// Structure to store iteration data
#[derive(Clone, Debug)]
struct IterationData {
    iteration: usize,
    delta: f64,
    point: Vec<f64>,
    function_value: f64,
    exploratory_steps: Vec<ExploratoryStep>,
    pattern_move: Option<PatternMove>,
}

#[derive(Clone, Debug)]
struct ExploratoryStep {
    variable_index: usize,
    direction: Vec<f64>,
    y_point: Vec<f64>,
    y_value: f64,
    y_plus_delta: Option<(Vec<f64>, f64)>,
    y_minus_delta: Option<(Vec<f64>, f64)>,
}

#[derive(Clone, Debug)]
struct PatternMove {
    direction: Vec<f64>,
    new_point: Vec<f64>,
    new_value: f64,
}

// Main application structure
struct HookeJeevesApp {
    function_choice: usize,
    initial_point_2d: [f64; 2],
    initial_point_3d: [f64; 3],
    epsilon: f64,
    initial_delta: f64,
    alpha: f64,
    iterations: Vec<IterationData>,
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

    fn optimize(&mut self) {
        self.iterations.clear();
        let mut x = self.get_initial_point();
        let mut delta = self.initial_delta;
        let n = self.get_dimension();
        let mut iteration_count = 0;

        while delta > self.epsilon {
            iteration_count += 1;
            let mut iter_data = IterationData {
                iteration: iteration_count,
                delta,
                point: x.clone(),
                function_value: self.evaluate_function(&x),
                exploratory_steps: Vec::new(),
                pattern_move: None,
            };

            let mut y = x.clone();
            let mut y_value = self.evaluate_function(&y);

            for j in 0..n {
                let mut direction = vec![0.0; n];
                direction[j] = 1.0;

                let mut step = ExploratoryStep {
                    variable_index: j,
                    direction: direction.clone(),
                    y_point: y.clone(),
                    y_value,
                    y_plus_delta: None,
                    y_minus_delta: None,
                };

                let mut y_plus = y.clone();
                y_plus[j] += delta;
                let f_plus = self.evaluate_function(&y_plus);

                if f_plus < y_value {
                    y = y_plus;
                    y_value = f_plus;
                    step.y_plus_delta = Some((y.clone(), f_plus));
                } else {
                    let mut y_minus = y.clone();
                    y_minus[j] -= delta;
                    let f_minus = self.evaluate_function(&y_minus);

                    if f_minus < y_value {
                        y = y_minus;
                        y_value = f_minus;
                        step.y_minus_delta = Some((y.clone(), f_minus));
                    }
                }
                iter_data.exploratory_steps.push(step);
            }

            if y_value < self.evaluate_function(&x) {
                let pattern_dir: Vec<f64> = y.iter()
                    .zip(x.iter())
                    .map(|(&a, &b)| a - b)
                    .collect();

                let new_point: Vec<f64> = y.iter()
                    .zip(pattern_dir.iter())
                    .map(|(&a, &b)| a + self.alpha * b)
                    .collect();

                let new_value = self.evaluate_function(&new_point);

                if new_value < y_value {
                    iter_data.pattern_move = Some(PatternMove {
                        direction: pattern_dir.clone(),
                        new_point: new_point.clone(),
                        new_value,
                    });
                    x = new_point;
                } else {
                    x = y;
                }
            } else {
                delta /= 2.0;
            }

            self.iterations.push(iter_data);

            if self.iterations.len() > 1 {
                let last = self.iterations.last().unwrap();
                let prev = self.iterations.iter().rev().nth(1).unwrap();
                
                let diff: f64 = last.point.iter()
                    .zip(prev.point.iter())
                    .map(|(&a, &b)| (a - b).abs())
                    .sum();

                if diff < self.epsilon && (last.function_value - prev.function_value).abs() < self.epsilon {
                    break;
                }
            }

            if iteration_count > 1000 {
                self.status_message = "Warning: Maximum iterations reached".to_string();
                break;
            }
        }

        if let Some(last) = self.iterations.last() {
            self.optimal_point = last.point.clone();
            self.optimal_value = last.function_value;
        }
        self.total_iterations = iteration_count;
        self.computation_done = true;
        self.status_message = "Optimization completed successfully!".to_string();
    }

    fn generate_level_lines_data(&self) -> (Vec<PlotPoints>, Vec<[f64; 2]>) {
        let mut level_lines = Vec::new();
        let mut path_points = Vec::new();

        if self.function_choice != 0 {
            return (level_lines, path_points);
        }

        // Generate optimization path
        for iter in &self.iterations {
            if iter.point.len() >= 2 {
                path_points.push([iter.point[0], iter.point[1]]);
            }
        }

        // Generate level lines for F1
        let levels = vec![8.0, 10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 25.0, 30.0];
        
        for level in levels {
            let mut points = Vec::new();
            
            // Use parametric approach for ellipses
            // F1 = (x1-3)^2 + (x2-2)^2 + 5
            // So level lines are circles centered at (3, 2)
            let radius_sq = level - 5.0;
            if radius_sq > 0.0 {
                let radius = (radius_sq as f64).sqrt();
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

    fn run_multiple_accuracies(&mut self) {
        let accuracies = [0.1, 0.01, 0.001];
        let mut results = String::new();
        
        for &eps in &accuracies {
            self.epsilon = eps;
            self.optimize();
            results.push_str(&format!("ε={}: Optimal={:.6}, Value={:.6}, Iterations={}\n",
                eps, self.optimal_point[0], self.optimal_value, self.total_iterations));
        }
        
        self.status_message = format!("Multiple accuracy test completed:\n{}", results);
    }

    fn run_multiple_initial_points(&mut self) {
        let test_points = if self.function_choice == 0 {
            vec![vec![0.0, 0.0], vec![2.0, 2.0], vec![-1.0, 3.0]]
        } else {
            vec![vec![0.0, 0.0, 0.0], vec![2.0, 1.0, 1.0]]
        };

        let mut results = String::new();
        let original_point = self.get_initial_point();

        for (_i, point) in test_points.iter().enumerate() {
            if self.function_choice == 0 {
                self.initial_point_2d = [point[0], point[1]];
            } else {
                self.initial_point_3d = [point[0], point[1], point[2]];
            }
            self.optimize();
            results.push_str(&format!("Point {:?}: Optimal={:.6}, Value={:.6}, Iterations={}\n",
                point, self.optimal_point[0], self.optimal_value, self.total_iterations));
        }

        if self.function_choice == 0 {
            self.initial_point_2d = [original_point[0], original_point[1]];
        } else {
            self.initial_point_3d = [original_point[0], original_point[1], original_point[2]];
        }
        
        self.status_message = format!("Multiple initial points test completed:\n{}", results);
    }
}

impl eframe::App for HookeJeevesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save Results").clicked() {
                        let mut output = String::new();
                        output.push_str("Hooke-Jeeves Optimization Results\n\n");
                        output.push_str(&format!("Function: F{}\n", self.function_choice + 1));
                        output.push_str(&format!("Optimal Point: {:?}\n", self.optimal_point));
                        output.push_str(&format!("Optimal Value: {}\n", self.optimal_value));
                        output.push_str(&format!("Total Iterations: {}\n", self.total_iterations));
                        
                        // Add iteration details
                        output.push_str("\nIterations:\n");
                        output.push_str("Iter | Delta | Point | F(Point)\n");
                        for iter in &self.iterations {
                            output.push_str(&format!(
                                "{} | {:.4} | {:?} | {:.6}\n",
                                iter.iteration, iter.delta, iter.point, iter.function_value
                            ));
                        }
                        
                        std::fs::write("optimization_results.txt", output).ok();
                        self.status_message = "Results saved to optimization_results.txt".to_string();
                    }
                });
            });
        });

        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            ui.heading("Hooke-Jeeves Method");
            ui.separator();
            
            ui.label("Select Function:");
            ui.radio_value(&mut self.function_choice, 0, "F₁(x) = -6X₁ - 4X₂ + X₁² + X₂² + 18");
            ui.radio_value(&mut self.function_choice, 1, "F₂(x) = 4X₁² + 3X₂² + X₃² + 4X₁X₂ - 2X₂X₃ - 16X₁ - 4X₃");
            
            ui.separator();
            ui.label("Parameters:");
            
            if self.function_choice == 0 {
                ui.horizontal(|ui| {
                    ui.label("Initial Point (X₁, X₂):");
                    ui.add(egui::DragValue::new(&mut self.initial_point_2d[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_2d[1]).speed(0.1));
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Initial Point:");
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[0]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[1]).speed(0.1));
                    ui.add(egui::DragValue::new(&mut self.initial_point_3d[2]).speed(0.1));
                });
            }
            
            ui.horizontal(|ui| {
                ui.label("Accuracy (ε):");
                ui.add(egui::DragValue::new(&mut self.epsilon).speed(0.001));
            });
            
            ui.horizontal(|ui| {
                ui.label("Initial Step (Δ):");
                ui.add(egui::DragValue::new(&mut self.initial_delta).speed(0.1));
            });
            
            ui.horizontal(|ui| {
                ui.label("Acceleration (α):");
                ui.add(egui::DragValue::new(&mut self.alpha).speed(0.1));
            });
            
            ui.separator();
            
            if ui.button("Run Optimization").clicked() {
                self.optimize();
            }
            
            if ui.button("Test Different ε Values").clicked() {
                self.run_multiple_accuracies();
            }
            
            if ui.button("Test Different Initial Points").clicked() {
                self.run_multiple_initial_points();
            }
            
            ui.separator();
            if !self.status_message.is_empty() {
                ui.label(&self.status_message);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.computation_done {
                ui.heading("Optimization Results");
                ui.separator();
                
                ui.label(format!("Function: F{}", self.function_choice + 1));
                ui.label(format!("Optimal Point: {:?}", self.optimal_point));
                ui.label(format!("Optimal Value: {:.6}", self.optimal_value));
                ui.label(format!("Total Iterations: {}", self.total_iterations));
                
                ui.separator();
                
                // Show iterations in a table
                ui.heading("Iteration Details (Table View)");
                ui.separator();
                
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    egui::Grid::new("iterations_table")
                        .num_columns(7)
                        .spacing([5.0, 2.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Header
                            ui.label("Iter");
                            ui.label("Delta");
                            ui.label("X₁");
                            ui.label("X₂");
                            if self.function_choice == 1 {
                                ui.label("X₃");
                            }
                            ui.label("F(X)");
                            ui.label("Pattern Move");
                            ui.end_row();
                            
                            // Data rows
                            for iter in &self.iterations {
                                ui.label(format!("{}", iter.iteration));
                                ui.label(format!("{:.4}", iter.delta));
                                if iter.point.len() > 0 {
                                    ui.label(format!("{:.4}", iter.point[0]));
                                } else {
                                    ui.label("-");
                                }
                                if iter.point.len() > 1 {
                                    ui.label(format!("{:.4}", iter.point[1]));
                                } else {
                                    ui.label("-");
                                }
                                if self.function_choice == 1 && iter.point.len() > 2 {
                                    ui.label(format!("{:.4}", iter.point[2]));
                                }
                                ui.label(format!("{:.6}", iter.function_value));
                                
                                if iter.pattern_move.is_some() {
                                    ui.label("Yes");
                                } else {
                                    ui.label("No");
                                }
                                ui.end_row();
                            }
                        });
                });
                
                ui.separator();
                
                // Show interactive plot for F1
                if self.function_choice == 0 {
                    if ui.checkbox(&mut self.show_plot, "Show Interactive Plot").changed() {
                        // Toggle plot visibility
                    }
                    
                    if self.show_plot {
                        let (level_lines, path_points) = self.generate_level_lines_data();
                        
                        ui.heading("Level Lines and Optimization Path");
                        
                        Plot::new("optimization_plot")
                            .view_aspect(1.0)
                            .width(500.0)
                            .height(400.0)
                            .show_axes([true, true])
                            .show_grid(true)
                            .label_formatter(|name, value| {
                                if name.is_empty() {
                                    format!("x: {:.2}\ny: {:.2}", value.x, value.y)
                                } else {
                                    format!("{}: x={:.2}, y={:.2}", name, value.x, value.y)
                                }
                            })
                            .show(ui, |plot_ui| {
                                // Draw level lines
                                for (i, line_points) in level_lines.into_iter().enumerate() {
                                    let line = Line::new(line_points)
                                        .color(egui::Color32::from_rgb(100, 100, 200))
                                        .width(1.0)
                                        .name(format!("Level {}", i));
                                    plot_ui.line(line);
                                }
                                
                                // Draw optimization path
                                if path_points.len() > 1 {
                                    let path_line = Line::new(
                                        PlotPoints::from(path_points.clone())
                                    )
                                    .color(egui::Color32::RED)
                                    .width(2.0)
                                    .name("Optimization Path");
                                    plot_ui.line(path_line);
                                    
                                    // Draw points
                                    let points: PlotPoints = path_points.iter()
                                        .map(|p| [p[0], p[1]])
                                        .collect();
                                    let points_plot = Points::new(points)
                                        .color(egui::Color32::RED)
                                        .radius(3.0)
                                        .name("Iterations");
                                    plot_ui.points(points_plot);
                                }
                                
                                // Draw start and end points
                                if !path_points.is_empty() {
                                    let start = vec![[path_points[0][0], path_points[0][1]]];
                                    let end = vec![[
                                        path_points[path_points.len()-1][0],
                                        path_points[path_points.len()-1][1]
                                    ]];
                                    
                                    plot_ui.points(
                                        Points::new(PlotPoints::from(start))
                                            .color(egui::Color32::GREEN)
                                            .radius(5.0)
                                            .name("Start")
                                    );
                                    plot_ui.points(
                                        Points::new(PlotPoints::from(end))
                                            .color(egui::Color32::BLUE)
                                            .radius(5.0)
                                            .name("End")
                                    );
                                }
                            });
                        
                        ui.horizontal(|ui| {
                            ui.label("🟢 Start point");
                            ui.label("🔵 End point");
                            ui.label("🔴 Optimization path");
                            ui.label("🔵 Level lines");
                        });
                    }
                }
                
                // Detailed exploratory steps in expandable section
                ui.separator();
                ui.collapsing("Detailed Exploratory Steps", |ui| {
                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        for iter in &self.iterations {
                            ui.collapsing(format!("Iteration {} (Δ={:.4})", iter.iteration, iter.delta), |ui| {
                                ui.label(format!("Point: {:?}", iter.point));
                                ui.label(format!("F(X) = {:.6}", iter.function_value));
                                
                                if !iter.exploratory_steps.is_empty() {
                                    ui.label("Exploratory Steps:");
                                    for step in &iter.exploratory_steps {
                                        ui.indent(format!("step_{}", step.variable_index), |ui| {
                                            ui.label(format!(
                                                "Variable j={}: y={:?}, F(y)={:.4}",
                                                step.variable_index, step.y_point, step.y_value
                                            ));
                                            if let Some((point, val)) = &step.y_plus_delta {
                                                ui.label(format!("  y+Δd: F={:.4}", val));
                                            }
                                            if let Some((point, val)) = &step.y_minus_delta {
                                                ui.label(format!("  y-Δd: F={:.4}", val));
                                            }
                                        });
                                    }
                                }
                                
                                if let Some(pm) = &iter.pattern_move {
                                    ui.label(format!(
                                        "Pattern Move: new_point={:?}, F={:.4}",
                                        pm.new_point, pm.new_value
                                    ));
                                }
                            });
                        }
                    });
                });
            } else {
                ui.heading("Hooke-Jeeves Optimization with Discrete Step");
                ui.separator();
                ui.label("Configure parameters in the left panel and click 'Run Optimization' to start.");
                ui.label("");
                ui.label("The algorithm will:");
                ui.label("1. Perform exploratory search along coordinate directions");
                ui.label("2. Attempt pattern move if exploratory search is successful");
                ui.label("3. Reduce step size if no improvement is found");
                ui.label("4. Continue until step size < ε");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1400.0, 900.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Hooke-Jeeves Optimizer",
        options,
        Box::new(|cc| Box::new(HookeJeevesApp::new(cc))),
    )
}