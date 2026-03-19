use rand::rng;
use rayon::prelude::*;

use crate::config::SimulationConfig;
use crate::model::run_single_simulation;
use crate::types::WeatherRecord;

#[derive(Debug, Clone)]
pub struct MonteCarloResults {
    pub emissions: Vec<f64>,
    pub costs: Vec<f64>,
}

pub fn monte_carlo(weather: &[WeatherRecord], config: &SimulationConfig) -> MonteCarloResults {
    let trial_results: Vec<_> = (0..config.n_trials)
        .into_par_iter()
        .map(|_| {
            let mut rng = rng();
            run_single_simulation(
                &mut rng,
                weather,
                config.mine_scale,
                config.renewable_ratio,
            )
        })
        .collect();

    let emissions = trial_results
        .iter()
        .map(|result| result.total_emissions)
        .collect();

    let costs = trial_results
        .iter()
        .map(|result| result.cost_per_kwh)
        .collect();

    MonteCarloResults { emissions, costs }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SummaryStats {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub min: f64,
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub max: f64,
}

pub fn summarize(values: &[f64]) -> SummaryStats {
    if values.is_empty() {
        return SummaryStats {
            count: 0,
            mean: 0.0,
            std_dev: 0.0,
            min: 0.0,
            p25: 0.0,
            p50: 0.0,
            p75: 0.0,
            max: 0.0,
        };
    }

    let count = values.len();
    let sum: f64 = values.iter().sum();
    let mean = sum / count as f64;

    let variance = values
        .iter()
        .map(|x| {
            let d = x - mean;
            d * d
        })
        .sum::<f64>()
        / count as f64;

    let std_dev = variance.sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    SummaryStats {
        count,
        mean,
        std_dev,
        min: sorted[0],
        p25: percentile_sorted(&sorted, 0.25),
        p50: percentile_sorted(&sorted, 0.50),
        p75: percentile_sorted(&sorted, 0.75),
        max: sorted[count - 1],
    }
}

fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    let pos = q * (n.saturating_sub(1)) as f64;

    let lower = pos.floor() as usize;
    let upper = pos.ceil() as usize;

    if lower == upper {
        sorted[lower]
    } else {
        let weight = pos - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}