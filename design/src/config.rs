#[derive(Debug, Clone, Copy)]
pub struct SimulationConfig {
    pub throughput_tpd: f64,
    pub renewable_ratio: f64,
    pub n_trials: usize,
}

impl SimulationConfig {
    pub fn new(throughput_tpd: f64, renewable_ratio: f64, n_trials: usize) -> Self {
        Self {
            throughput_tpd,
            renewable_ratio,
            n_trials,
        }
    }
}
