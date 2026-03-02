use crate::models::{Iteration, OptimizationResult};
use std::cell::Cell;

// Вспомогательная структура для подсчета вызовов целевой функции
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

// Метод дихотомии
pub fn dichotomy_method(
    a_init: f64,
    b_init: f64,
    eps: f64,
    l: f64,
    f: &Func,
    is_max: bool,
) -> Result<OptimizationResult, String> {
    if eps <= 0.0 || l <= 0.0 || 2.0 * eps >= l {
        return Err("Условие 2*eps < l не выполнено!".to_string());
    }

    let mut a = a_init;
    let mut b = b_init; // Границы интервала
    let mut history = Vec::new(); // История итераций
    let mut k = 1; // Номер итерации

    while (b - a) > l {
        let lambda = (a + b) / 2.0 - eps;
        let mu = (a + b) / 2.0 + eps;

        let f_lambda = f.eval(lambda);
        let f_mu = f.eval(mu);

        history.push(Iteration {
            k,
            a,
            b,
            lambda,
            mu,
            f_lambda,
            f_mu,
        });

        let condition = if is_max {
            f_lambda < f_mu
        } else {
            f_lambda > f_mu
        };

        if condition {
            a = lambda;
        } else {
            b = mu;
        }

        k += 1;
    }

    let x_opt = (a + b) / 2.0;
    Ok(OptimizationResult {
        x_opt,
        f_opt: f.eval(x_opt),
        history,
        fn_calls: f.calls.get(),
    })
}

/// Метод золотого сечения
pub fn golden_ratio_method(
    a_init: f64,
    b_init: f64,
    _eps: f64, // eps используется только для проверки условия, но не в самом алгоритме
    l: f64,
    f: &Func,
    is_max: bool,
) -> Result<OptimizationResult, String> {
    if _eps <= 0.0 || l <= 0.0 || 2.0 * _eps >= l {
        return Err("Условие 2*eps < l не выполнено!".to_string());
    }

    let alpha = 0.618f64;
    let mut a = a_init;
    let mut b = b_init;
    let mut history = Vec::new();
    let mut k = 1;

    let mut lambda = a + (1.0 - alpha) * (b - a);
    let mut mu = a + alpha * (b - a);

    let mut f_lambda = f.eval(lambda);
    let mut f_mu = f.eval(mu);

    while (b - a) > l {
        history.push(Iteration {
            k,
            a,
            b,
            lambda,
            mu,
            f_lambda,
            f_mu,
        });

        let condition = if is_max {
            f_lambda < f_mu
        } else {
            f_lambda > f_mu
        };

        // ТУТ есть вопросы. ПЕРЕПРОВЕРИТЬ
        // ->  https://github.com/Dickhat/TOPY_lab1_var3/blob/99535e4d5fde764c46387981f5969624ebb1daa5/src/main.rs#L80
        // У данича немного по-другому: он не обновляет f_lambda и f_mu в обоих ветках, а только в одной.
        // Но тогда в следующей итерации будет использоваться устаревшее значение.
        // Я решил обновлять оба, чтобы не было путаницы, но возможно это не совсем оптимально.
        // В любом случае, так работает корректно.
        if condition {
            a = lambda;
            lambda = mu;
            f_lambda = f_mu; // TODO этого у данича нет
            mu = a + alpha * (b - a);
            f_mu = f.eval(mu);
        } else {
            b = mu;
            mu = lambda;
            f_mu = f_lambda;
            lambda = a + (1.0 - alpha) * (b - a);
            f_lambda = f.eval(lambda);
        }
        k += 1;
    }

    let x_opt = (a + b) / 2.0;
    Ok(OptimizationResult {
        x_opt,
        f_opt: f.eval(x_opt),
        history,
        fn_calls: f.calls.get(),
    })
}

/// Вспомогательная функция для генерации чисел Фибоначчи (итеративно)
fn get_fibonacci_n(limit: f64) -> Vec<f64> {
    let mut fibs = vec![1.0, 1.0];
    while *fibs.last().unwrap() < limit {
        let next = fibs[fibs.len() - 1] + fibs[fibs.len() - 2];
        fibs.push(next);
    }
    fibs
}

/// Метод Фибоначчи
pub fn fibonacci_method(
    a_init: f64,
    b_init: f64,
    eps: f64,
    l: f64,
    f: &Func,
    is_max: bool,
) -> Result<OptimizationResult, String> {
    if eps <= 0.0 || l <= 0.0 || 2.0 * eps >= l {
        return Err("Eps или l не корректно заданы".to_string());
    }
    let mut a = a_init;
    let mut b = b_init;
    let mut history = Vec::new();

    let fibs = get_fibonacci_n((b - a) / l);
    let n = fibs.len() - 1; // количество итераций

    let mut lambda = a + (fibs[n - 2] / fibs[n]) * (b - a);
    let mut mu = a + (fibs[n - 1] / fibs[n]) * (b - a);

    let mut f_lambda = f.eval(lambda);
    let mut f_mu = f.eval(mu);

    for k in 1..n {
        history.push(Iteration {
            k,
            a,
            b,
            lambda,
            mu,
            f_lambda,
            f_mu,
        });

        let condition = if is_max {
            f_lambda < f_mu
        } else {
            f_lambda > f_mu
        };

        if condition {
            a = lambda;
            lambda = mu;
            f_lambda = f_mu;
            mu = a + (fibs[n - k - 1] / fibs[n - k]) * (b - a);
            if k < n - 1 {
                f_mu = f.eval(mu);
            }
        } else {
            b = mu;
            mu = lambda;
            f_mu = f_lambda;
            lambda = a + (fibs[n - k - 2] / fibs[n - k]) * (b - a);
            if k < n - 1 {
                f_lambda = f.eval(lambda);
            }
        }
    }

    // Финальный шаг n-1 с использованием eps
    let lambda_n = mu;
    let mu_n = lambda_n + eps;
    let f_ln = f.eval(lambda_n);
    let f_mn = f.eval(mu_n);

    let condition = if is_max { f_ln < f_mn } else { f_ln > f_mn };
    if condition {
        a = lambda_n;
    } else {
        b = mu_n;
    }

    let x_opt = (a + b) / 2.0;
    Ok(OptimizationResult {
        x_opt,
        f_opt: f.eval(x_opt),
        history,
        fn_calls: f.calls.get(),
    })
}
