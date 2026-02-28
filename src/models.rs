// В будущем это может быть в файле src/models.rs
#[derive(Clone, Debug)]
pub struct Iteration {
    pub k: usize,
    pub a: f64,
    pub b: f64,
    pub lambda: f64,
    pub mu: f64,
    pub f_lambda: f64,
    pub f_mu: f64,
}

pub struct OptimizationResult {
    pub x_opt: f64,
    pub f_opt: f64,
    pub history: Vec<Iteration>,
    pub fn_calls: usize,
}