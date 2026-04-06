use rand::rng;
use rayon::prelude::*;

use crate::config::SimulationConfig;
use crate::model::run_single_simulation;
use crate::types::WeatherRecord;

// Social acceptance (Appendix K)
const SA_BASE: f64 = 0.50;
const SA_WIND: f64 = 0.30;    // bonus from wind fraction
const SA_COST: f64 = 0.15;    // spread from stochastic diesel cost

#[derive(Debug, Clone)]
pub struct MonteCarloResults {
    pub emissions: Vec<f64>,
    pub costs: Vec<f64>,
    pub social_acceptance: Vec<f64>,
}

pub fn monte_carlo(
    weather: &[WeatherRecord],
    config: &SimulationConfig,
    _population: f64,
) -> MonteCarloResults {
    let trial_results: Vec<_> = (0..config.n_trials)
        .into_par_iter()
        .map(|_| {
            let mut rng = rng();
            run_single_simulation(
                &mut rng,
                weather,
                config.throughput_tpd,
                config.renewable_ratio,
            )
        })
        .collect();

    let emissions: Vec<f64> = trial_results.iter().map(|r| r.emissions_intensity).collect();
    let costs: Vec<f64> = trial_results.iter().map(|r| r.cost_per_year).collect();

    let cost_mean = costs.iter().sum::<f64>() / costs.len() as f64;
    let cost_std = {
        let var = costs.iter().map(|c| (c - cost_mean).powi(2)).sum::<f64>() / costs.len() as f64;
        var.sqrt()
    };

    let social_acceptance = trial_results
        .iter()
        .map(|r| {
            let cost_z = if cost_std > 0.0 {
                (cost_mean - r.cost_per_year) / cost_std
            } else {
                0.0
            };
            (SA_BASE + SA_WIND * r.wind_fraction + SA_COST * cost_z).clamp(0.0, 1.0)
        })
        .collect();

    MonteCarloResults {
        emissions,
        costs,
        social_acceptance,
    }
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
