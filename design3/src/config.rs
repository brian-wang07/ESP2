#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub mine_scale: f64,
    pub renewable_ratio: f64,
    pub n_trials: usize,
}

impl SimulationConfig {
    pub fn new(mine_scale: f64, renewable_ratio: f64, n_trials: usize) -> Self {
        Self {
            mine_scale,
            renewable_ratio,
            n_trials,
        }
    }
}