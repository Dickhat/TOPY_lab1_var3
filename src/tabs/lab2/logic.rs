use super::models::{HJIterationData, HJResult, DirectionStep};

/// Метод Хука-Джевса с дискретным шагом
pub fn hooke_jeeves<F>(
    mut x: Vec<f64>,
    mut delta: f64,
    delta_min: f64,
    _epsilon: f64,
    f: F,
) -> Result<HJResult, String>
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    let mut iterations = Vec::new();
    let mut function_calls = 0;
    let mut iteration_count = 0;
    let max_iterations = 10000;

    let mut f_current = f(&x);
    function_calls += 1;

    while delta > delta_min && iteration_count < max_iterations {
        iteration_count += 1;

        let mut improved = false;
        let mut direction_steps = Vec::new();

        // Исследующий поиск: пробуем изменять каждую координату
        for j in 0..n {
            let mut best_point = x.clone();
            let mut best_f = f_current;

            // Пробуем положительное направление
            let mut point_plus = x.clone();
            point_plus[j] += delta;
            let f_plus = f(&point_plus);
            function_calls += 1;

            // Пробуем отрицательное направление
            let mut point_minus = x.clone();
            point_minus[j] -= delta;
            let f_minus = f(&point_minus);
            function_calls += 1;

            let mut accepted_plus = false;

            if f_plus < best_f {
                best_f = f_plus;
                best_point = point_plus.clone();
                accepted_plus = true;
                improved = true;
            }

            if f_minus < best_f {
                best_f = f_minus;
                best_point = point_minus.clone();
                accepted_plus = false;
                improved = true;
            }

            // Запоминаем этап (даже если улучшение не найдено)
            direction_steps.push(DirectionStep {
                var_index: j,
                direction: if accepted_plus { 1.0 } else { -1.0 },
                step_size: delta,
                point_plus: point_plus.clone(),
                f_plus,
                point_minus: point_minus.clone(),
                f_minus,
                accepted_plus,
            });

            // Если найдено улучшение, сохраняем новую точку
            if best_f < f_current {
                x = best_point;
                f_current = best_f;
            }
        }

        // Сохраняем итерацию
        iterations.push(HJIterationData {
            iteration: iteration_count,
            current_point: x.clone(),
            f_value: f_current,
            delta,
            direction_steps,
        });

        // Если не было улучшений, уменьшаем шаг
        if !improved {
            delta /= 2.0;
        }
    }

    if iteration_count >= max_iterations {
        return Err("Максимально количество итераций достигнуто".to_string());
    }

    Ok(HJResult {
        x_opt: x,
        f_opt: f_current,
        iterations,
        num_iterations: iteration_count,
        function_calls,
    })
}
