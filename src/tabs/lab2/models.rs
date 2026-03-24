#[derive(Clone, Debug)]
pub struct HJIterationData {
    pub iteration: usize,
    pub current_point: Vec<f64>,
    pub f_value: f64,
    pub delta: f64,
    
    // Информация о направлениях и шагах в этой итерации
    pub direction_steps: Vec<DirectionStep>,
}

#[derive(Clone, Debug)]
pub struct DirectionStep {
    pub var_index: usize,        // Индекс переменной (j)
    pub direction: f64,          // Направление (dj) = +1.0 или -1.0
    pub step_size: f64,          // Величина шага (λj)
    pub point_plus: Vec<f64>,    // Точка с направлением +dj
    pub f_plus: f64,             // Значение функции в point_plus
    pub point_minus: Vec<f64>,   // Точка с направлением -dj
    pub f_minus: f64,            // Значение функции в point_minus
    pub accepted_plus: bool,     // Был ли выбран шаг в положительном направлении
}

pub struct HJResult {
    pub x_opt: Vec<f64>,
    pub f_opt: f64,
    pub iterations: Vec<HJIterationData>,
    pub num_iterations: usize,
    pub function_calls: usize,
}
