use std::io::{self, Write};
use plotters::prelude::*;

fn dichotomy_method<F>(mut a: f32, mut b: f32, eps: f32, l: f32, function: F, extremum_max: bool) -> Result<(f32, f32), String>
where 
    F: Fn(f32) -> f32
{
    if eps <= 0.0 || l <= 0.0
    {
        return Err("Eps или l не корректно заданы".to_string());
    }

    let mut k = 1; /* Итераций */
    let mut lamda: f32;
    let mut mu: f32;

    println!("{:<3} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10} | {:>10}", 
                "K",  "a_k",  "b_k", "lamda", "mu", "F(lamda)", "F(mu)");

    while b - a > l
    {
        lamda = (a + b) / 2.0 - eps;
        mu = (a + b) / 2.0 + eps;

        if extremum_max {
            if function(lamda) < function(mu) { a = lamda; } 
            else { b = mu; }
        } 
        else
        {
            if function(lamda) > function(mu) { a = lamda; } 
            else { b = mu; }
        }

        println!("{:<3} | {:>8.4} | {:>8.4} | {:>8.4} | {:>8.4} | {:>10.4} | {:>10.4}", k, a, b, lamda, mu, function(lamda), function(mu));
        k += 1;
    }

    Ok((a, b))
}

fn golden_ratio_method<F>(mut a: f32, mut b: f32, eps: f32, l: f32, function: F, extremum_max: bool) -> Result<(f32, f32), String>
where
    F: Fn(f32) -> f32
{
    if eps <= 0.0 || l <= 0.0
    {
        return Err("Eps или l не корректно заданы".to_string());
    }

    let alpha = 0.618;
    let mut k = 1; /* Итераций */
    let mut lamda: f32 = a + (1.0 - alpha) * (b - a);
    let mut mu: f32 = a + alpha * (b - a);

    let mut fn_lamda: f32 = function(lamda);
    let mut fn_mu: f32 = function(mu);

    println!("{:<3} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10} | {:>10}", 
                "K",  "a_k",  "b_k", "lamda", "mu", "F(lamda)", "F(mu)");

    while b - a > l
    {
        if extremum_max {
            if fn_lamda < fn_mu { 
                a = lamda; 
                lamda = mu;
                mu = a + alpha * (b - a);
                fn_mu = function(mu);
            } 
            else { 
                b = mu;
                mu = lamda;
                lamda = a + (1.0 - alpha) * (b - a); 
                fn_lamda = function(lamda);
            }
        } 
        else
        {
            if fn_lamda > fn_mu { 
                a = lamda; 
                lamda = mu;
                mu = a + alpha * (b - a);
                fn_mu = function(mu);
            } 
            else { 
                b = mu;
                mu = lamda;
                lamda = a + (1.0 - alpha) * (b - a); 
                fn_lamda = function(lamda);
            }
        }

        println!("{:<3} | {:>8.4} | {:>8.4} | {:>8.4} | {:>8.4} | {:>10.4} | {:>10.4}", k, a, b, lamda, mu, function(lamda), function(mu));
        k += 1;
    }

    Ok((a, b))
}

fn fibonachi_method<F>(mut a: f32, mut b: f32, eps: f32, l: f32, function: F, extremum_max: bool) -> Result<(f32, f32), String>
where
    F: Fn(f32) -> f32
{
    if eps <= 0.0 || l <= 0.0
    {
        return Err("Eps или l не корректно заданы".to_string());
    }

    let mut k = 1; /* Итераций */
    let mut lamda: f32;
    let mut mu: f32;

    println!("{:<3} | {:>8} | {:>8} | {:>8} | {:>8} | {:>10} | {:>10}", 
                "K",  "a_k",  "b_k", "lamda", "mu", "F(lamda)", "F(mu)");

    while b - a > l
    {
        lamda = (a + b) / 2.0 - eps;
        mu = (a + b) / 2.0 + eps;

        if extremum_max {
            if function(lamda) < function(mu) { a = lamda; } 
            else { b = mu; }
        } 
        else
        {
            if function(lamda) > function(mu) { a = lamda; } 
            else { b = mu; }
        }

        println!("{:<3} | {:>8.4} | {:>8.4} | {:>8.4} | {:>8.4} | {:>10.4} | {:>10.4}", k, a, b, lamda, mu, function(lamda), function(mu));
        k += 1;
    }

    Ok((a, b))
}

fn draw_result_plot_fn1(a_orig: f32, b_orig: f32, res_a: f32, res_b: f32) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plot1.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Определяем диапазон функции для отрисовки
    let f = |x: f32| 3.0 * x - x.powi(3);

    let mut chart = ChartBuilder::on(&root)
        .caption("MAX: f(x) = 3x - x^3", ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(a_orig..b_orig, -5.0f32..5.0f32)?;

    chart.configure_mesh().draw()?;

    // 1. Отрисовка самой функции
    chart.draw_series(LineSeries::new(
        (-300..300).map(|x| a_orig + (b_orig - a_orig) * (x as f32 / 600.0 + 0.5)).map(|x| (x, f(x))),
        BLUE,
    ))?
    .label("f(x)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // 2. Подсветка найденного интервала (вертикальная область)
    chart.draw_series(std::iter::once(Rectangle::new(
        [(res_a, -5.0), (res_b, 5.0)],
        RED.mix(0.2).filled(),
    )))?
    .label("Найденный интервал");

    // 3. Линия в центре результата
    let final_x = (res_a + res_b) / 2.0;
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(final_x, -5.0), (final_x, 5.0)],
        RED.stroke_width(2),
    )))?;

    chart.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    
    println!("\nГрафик сохранен в файл 'plot1.png'");
    Ok(())
}

fn draw_result_plot_fn2(a_orig: f32, b_orig: f32, res_a: f32, res_b: f32) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("plot2.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    // Определяем диапазон функции для отрисовки
    let f = |x: f32| (9.0 - x.powi(2)) / (x.powi(2) + 2.0*x + 3.0);

    let mut chart = ChartBuilder::on(&root)
        .caption("MIN: f(x) = (9 - x^2) / (x^2 + 2x + 3)", ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(a_orig..b_orig, -5.0f32..5.0f32)?;

    chart.configure_mesh().draw()?;

    // 1. Отрисовка самой функции
    chart.draw_series(LineSeries::new(
        (-300..300).map(|x| a_orig + (b_orig - a_orig) * (x as f32 / 600.0 + 0.5)).map(|x| (x, f(x))),
        BLUE,
    ))?
    .label("f(x)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // 2. Подсветка найденного интервала (вертикальная область)
    chart.draw_series(std::iter::once(Rectangle::new(
        [(res_a, -10.0), (res_b, 10.0)],
        RED.mix(0.2).filled(),
    )))?
    .label("Найденный интервал");

    // 3. Линия в центре результата
    let final_x = (res_a + res_b) / 2.0;
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(final_x, -10.0), (final_x, 10.0)],
        RED.stroke_width(2),
    )))?;

    chart.configure_series_labels().background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    
    println!("\nГрафик сохранен в файл 'plot2.png'");
    Ok(())
}

/* Сбор входных данных  */
fn input_data_catch() -> (f32, f32, f32, f32)
{
    let mut input = String::new();

    print!("Введите значение a: ");
    io::stdout().flush().unwrap(); 

    io::stdin()
        .read_line(&mut input)
        .expect("Не удалось прочитать строку");

    let a: f32 = input.trim().parse().unwrap_or(10.0);
    input.clear();

    print!("Введите значение b: ");
    io::stdout().flush().unwrap(); 

    io::stdin()
        .read_line(&mut input)
        .expect("Не удалось прочитать строку");

    let b: f32 = input.trim().parse().unwrap_or(10.0);
    input.clear();

    print!("Введите значение eps: ");
    io::stdout().flush().unwrap(); 

    io::stdin()
        .read_line(&mut input)
        .expect("Не удалось прочитать строку");

    let eps: f32 = input.trim().parse().unwrap_or(0.1);
    input.clear();

    print!("Введите значение mu: ");
    io::stdout().flush().unwrap(); 

    io::stdin()
        .read_line(&mut input)
        .expect("Не удалось прочитать строку");

    let mu: f32 = input.trim().parse().unwrap_or(0.2);
    input.clear();

    (a, b, eps, mu)
}

fn main() {
    let f1_max = |x: f32| -> f32 {return 3.0 * x - x*x*x;};
    let f2_min = |x: f32| -> f32 {return (9.0 - x*x) / (x*x + 2.0*x + 3.0);};

    let (a, b, eps, mu) = input_data_catch();

    match golden_ratio_method(a, b, eps, mu, f1_max, true) {
        Ok((res_a, res_b)) => {
            println!("Отрезок [{:?}, {:?}]", res_a, res_b);
            if let Err(e) = draw_result_plot_fn1(a, b, res_a, res_b) {
                println!("Не удалось создать график: {}", e);
            }
        },
        Err(error) => println!("Ошибка: {:?}", error)
    };

    match golden_ratio_method(a, b, eps, mu, f2_min, false) {
        Ok((res_a, res_b)) => {
            println!("Отрезок [{:?}, {:?}]", res_a, res_b);
            if let Err(e) = draw_result_plot_fn2(a, b, res_a, res_b) {
                println!("Не удалось создать график: {}", e);
            }
        },
        Err(error) => println!("Ошибка: {:?}", error)
    };
}
