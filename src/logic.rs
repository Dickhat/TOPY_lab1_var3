use crate::models::{Iteration, OptimizationResult};
use std::cell::Cell;

// Обертка для подсчета вызовов функции
pub struct Func<'a> {
    pub f: &'a dyn Fn(f64) -> f64,
    pub calls: Cell<usize>,
}

impl<'a> Func<'a> {
    pub fn eval(&self, x: f64) -> f64 {
        self.calls.set(self.calls.get() + 1);
        (self.f)(x)
    }
}

pub fn dichotomy(a_init: f64, b_init: f64, eps: f64, l: f64, f: &Func, is_max: bool) -> OptimizationResult {
    let mut a = a_init;
    let mut b = b_init;
    let mut history = Vec::new();
    let mut k = 1;

    while (b - a) > l {
        let lambda = (a + b) / 2.0 - eps;
        let mu = (a + b) / 2.0 + eps;
        let f_l = f.eval(lambda);
        let f_m = f.eval(mu);

        history.push(Iteration { k, a, b, lambda, mu, f_lambda: f_l, f_mu: f_m });

        if if is_max { f_l < f_m } else { f_l > f_m } { a = lambda; } else { b = mu; }
        k += 1;
    }

    let x_opt = (a + b) / 2.0;
    OptimizationResult { x_opt, f_opt: f.eval(x_opt), history, fn_calls: f.calls.get() }
}

// Здесь же добавьте функции для Золотого сечения и Фибоначчи