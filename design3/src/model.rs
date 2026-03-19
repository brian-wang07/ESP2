use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::types::WeatherRecord;

pub const RHO: f64 = 1.225;
pub const ROTOR_AREA: f64 = 3959.0;
pub const CP: f64 = 0.4;
pub const TURBINE_RATED_POWER_KW: f64 = 2300.0;

pub const FUEL_CONSUMPTION: f64 = 0.27; // L/kWh
pub const CO2_PER_LITER: f64 = 2.68;

pub const K_AVG: f64 = 1.667; // MW/km^2 baseline

pub const DIESEL_PRICE_INIT: f64 = 1.5; // CAD/L
pub const DIESEL_MU: f64 = 0.0;
pub const DIESEL_SIGMA: f64 = 0.2;


#[derive(Debug, Clone, Copy)]
pub struct TrialOutput {
    pub total_emissions: f64,
    pub cost_per_kwh: f64,
}


/// Equivalent to:
/// power = 0.5 * rho * ROTOR_AREA * cp * v^3 * n_turbines
/// power_kw = power / 1000
/// return min(power_kw, n_turbines * TURBINE_RATED_POWER_KW)
pub fn wind_power(v: f64, rho: f64, cp: f64, n_turbines: usize) -> f64 {
    let power_w = 0.5 * rho * ROTOR_AREA * cp * v.powi(3) * n_turbines as f64;
    let power_kw = power_w / 1000.0;

    let max_power = n_turbines as f64 * TURBINE_RATED_POWER_KW;
    power_kw.min(max_power)
}

/// Independent diesel price sampling.
///
/// This replaces the time-dependent GBM path in Python.
/// Instead of evolving P_t over time, each timestep draws:
///
/// price = DIESEL_PRICE_INIT * exp((mu - 0.5*sigma^2) + sigma * z)
/// where z ~ N(0, 1)
///
/// This keeps prices positive and preserves the lognormal-style sampling,
/// but removes all time dependence.
pub fn sample_diesel_price<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    let z: f64 = Normal::new(0.0, 1.0)
        .expect("valid standard normal")
        .sample(rng);

    DIESEL_PRICE_INIT * ((DIESEL_MU - 0.5 * DIESEL_SIGMA.powi(2)) + DIESEL_SIGMA * z).exp()
}

/// Same Python logic:
/// demand_mw = mine_scale * K_AVG
/// target_kwh = renewable_ratio * demand_mw * 1000
/// mean_v = mean WIND_SPEED
/// avg_power_per_turbine_kw = min(0.5 * rho * A * cp * mean_v^3 / 1000, rated)
/// n_turbines = ceil(target_kwh / avg_power_per_turbine_kw)
pub fn estimate_required_turbines(
    weather: &[WeatherRecord],
    mine_scale: f64,
    renewable_ratio: f64,
    rho: f64,
    cp: f64,
) -> usize {
    let demand_mw = mine_scale * K_AVG;
    let target_kwh = renewable_ratio * demand_mw * 1000.0;

    let mut sum_v = 0.0;
    let mut count = 0usize;

    for record in weather {
        if let Some(v) = record.wind_speed {
            if v.is_finite() {
                sum_v += v;
                count += 1;
            }
        }
    }

    if count == 0 {
        return 1;
    }

    let mean_v = sum_v / count as f64;

    let mut avg_power_per_turbine_kw =
        0.5 * rho * ROTOR_AREA * cp * mean_v.powi(3) / 1000.0;

    avg_power_per_turbine_kw = avg_power_per_turbine_kw.min(TURBINE_RATED_POWER_KW);

    if avg_power_per_turbine_kw <= 0.0 {
        return 1;
    }

    let n_turbines = (target_kwh / avg_power_per_turbine_kw).ceil() as usize;
    n_turbines.max(1)
}

/// Runs one full simulation trial.
///
/// This mirrors the Python `run_single_simulation(...)`, except:
/// - weather sampling is done by random indexing with replacement
/// - diesel price is independently sampled each timestep
pub fn run_single_simulation<R: Rng + ?Sized>(
    rng: &mut R,
    weather: &[WeatherRecord],
    mine_scale: f64,
    renewable_ratio: f64,
) -> TrialOutput {
    let mut total_energy_kwh = 0.0;
    let mut total_cost = 0.0;
    let mut total_emissions = 0.0;

    let cp = Normal::new(CP, 0.05)
        .expect("valid cp distribution")
        .sample(rng);

    let rho = Normal::new(RHO, 0.05)
        .expect("valid rho distribution")
        .sample(rng);

    let n_turbines = estimate_required_turbines(weather, mine_scale, renewable_ratio, rho, cp);

    let demand_mw = mine_scale * K_AVG;
    let demand_kwh = demand_mw * 1000.0;
    let renewable_target = renewable_ratio * demand_kwh;

    let wind_perturbation = Normal::new(1.0, 0.1).expect("valid wind perturbation distribution");

    for _ in 0..weather.len() {
        let idx = rng.random_range(0..weather.len());
        let row = &weather[idx];

        let Some(mut wind_speed) = row.wind_speed else {
            continue;
        };

        if !wind_speed.is_finite() {
            continue;
        }

        wind_speed *= wind_perturbation.sample(rng);

        let wind_kwh = wind_power(wind_speed, rho, cp, n_turbines);
        let wind_used = wind_kwh.min(renewable_target);
        let diesel_needed = demand_kwh - wind_used;

        let diesel_price = sample_diesel_price(rng);
        let diesel_liters = diesel_needed * FUEL_CONSUMPTION;
        let cost_diesel = diesel_liters * diesel_price;

        let cost_wind = wind_used * 0.05;
        let emissions = diesel_liters * CO2_PER_LITER;

        total_energy_kwh += demand_kwh;
        total_cost += cost_wind + cost_diesel;
        total_emissions += emissions;
    }

    let cost_per_kwh = if total_energy_kwh > 0.0 {
        total_cost / total_energy_kwh
    } else {
        0.0
    };

    TrialOutput {
        total_emissions,
        cost_per_kwh,
    }
}