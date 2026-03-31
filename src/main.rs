use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};

fn main() {
    // 1. Setup for Web (WASM)
    #[cfg(target_arch = "wasm32")]
    {
        // This ensures panics give readable errors in the browser console
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();
        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "the_canvas_id", // Make sure your index.html has a <canvas id="the_canvas_id"></canvas>
                    web_options,
                    Box::new(|_cc| Ok(Box::<OptimizerApp>::default())),
                )
                .await
                .expect("failed to start eframe");
        });
    }

    // 2. Setup for Native (so `cargo run` won't fail!)
    #[cfg(not(target_arch = "wasm32"))]
    {
        let native_options = eframe::NativeOptions::default();
        let _ = eframe::run_native(
            "Метод Хука-Дживса",
            native_options,
            Box::new(|_cc| Ok(Box::<OptimizerApp>::default())),
        );
    }
}

struct OptimizerApp {
    eps_val: f64,
    x_start: [f64; 2],
    history: Vec<[f64; 2]>,
    k: usize,
    j: usize,
}

impl Default for OptimizerApp {
    fn default() -> Self {
        Self {
            eps_val: 0.01,
            x_start: [1.0, 1.0],
            history: Vec::new(),
            k: 0,
            j: 0,
        }
    }
}

fn f(x: [f64; 2]) -> f64 {
    -6.0 * x[0] - 4.0 * x[1] + x[0] * x[0] + x[1] * x[1] + 18.0
}

fn golden_section<F>(f: F, p: [f64; 2], d: [f64; 2], eps: f64) -> f64 
where F: Fn([f64; 2]) -> f64 {
    let (mut a, mut b) = (-10.0, 10.0);
    let k_phi = 0.618;
    let get_val = |l: f64| f([p[0] + l * d[0], p[1] + l * d[1]]);
    
    let mut x1 = b - k_phi * (b - a);
    let mut x2 = a + k_phi * (b - a);
    let (mut f1, mut f2) = (get_val(x1), get_val(x2));

    while (b - a).abs() > eps {
        if f1 < f2 { b = x2; x2 = x1; f2 = f1; x1 = b - k_phi * (b - a); f1 = get_val(x1); }
        else { a = x1; x1 = x2; f1 = f2; x2 = a + k_phi * (b - a); f2 = get_val(x2); }
    }
    (a + b) / 2.0
}

impl eframe::App for OptimizerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Метод Хука-Дживса (WASM)");
            
            ui.horizontal(|ui| {
                ui.label("Eps:");
                ui.add(egui::DragValue::new(&mut self.eps_val).speed(0.001));
                ui.label("X1:");
                ui.add(egui::DragValue::new(&mut self.x_start[0]));
                ui.label("X2:");
                ui.add(egui::DragValue::new(&mut self.x_start[1]));
            });

            if ui.button("Рассчитать").clicked() {
                self.run_calc();
            }

            ui.label(format!("Итераций k: {}, Шагов j: {}", self.k, self.j));

            let plot = Plot::new("map").data_aspect(1.0).height(400.0);
            plot.show(ui, |plot_ui| {
                let pts: Vec<[f64; 2]> = self.history.iter().cloned().collect();
                plot_ui.line(Line::new(PlotPoints::from(pts.clone())).color(egui::Color32::LIGHT_BLUE));
                plot_ui.points(Points::new(PlotPoints::from(pts)).radius(4.0).color(egui::Color32::WHITE));
            });
        });
    }
}

impl OptimizerApp {
    fn run_calc(&mut self) {
        let mut x = self.x_start;
        self.history = vec![x];
        self.k = 0; self.j = 0;

        for _ in 0..100 {
            self.k += 1;
            let mut y = x;
            for i in 0..2 {
                self.j += 1;
                let mut d = [0.0, 0.0]; d[i] = 1.0;
                y[i] += golden_section(f, y, d, self.eps_val);
            }
            let x_next = y;
            let dist = ((x_next[0]-x[0]).powi(2) + (x_next[1]-x[1]).powi(2)).sqrt();
            self.history.push(x_next);
            if dist < self.eps_val { break; }

            let s = [x_next[0]-x[0], x_next[1]-x[1]];
            let l_bar = golden_section(f, x_next, s, self.eps_val);
            x = x_next;
            // Обновление y_1 для следующего шага
            // y = [x_next[0] + l_bar*s[0], x_next[1] + l_bar*s[1]]; 
        }
    }
}
