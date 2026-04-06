use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::types::WeatherRecord;

// From Appendix N: Cp ~ N(0.2901, 0.185^2)
pub const CP_MEAN: f64 = 0.2901;
pub const CP_STD: f64 = 0.085;

// From Appendix N: rho ~ N(1.33, 0.08^2), derived from ideal gas law on temperature data
pub const RHO_MEAN: f64 = 1.33;
pub const RHO_STD: f64 = 0.08;

pub const ROTOR_AREA: f64 = 3959.0;
pub const TURBINE_RATED_POWER_KW: f64 = 2300.0;

pub const FUEL_CONSUMPTION: f64 = 0.27; // L/kWh (diesel generator efficiency)
pub const CO2_PER_LITER: f64 = 2.68;

pub const ENERGY_INTENSITY: f64 = 12.0; // kWh/t (electrical demand per tonne)
pub const EMISSION_INTENSITY: f64 = 49.0; // kg CO2/t at 0% renewables (total site emissions)
pub const WIND_COST: f64 = 0.131; // CAD/kWh

// From Appendix N: initial diesel price as of week of March 17, 2026
pub const DIESEL_PRICE_INIT: f64 = 2.122; // CAD/L
pub const DIESEL_MU: f64 = 0.0;
pub const DIESEL_SIGMA: f64 = 0.2;


#[derive(Debug, Clone, Copy)]
pub struct TrialOutput {
    pub emissions_intensity: f64, // kg CO2/t
    pub cost_per_year: f64,
    pub n_turbines: usize,
    pub avg_diesel_kw: f64,
    pub wind_fraction: f64,
}


pub fn wind_power(v: f64, rho: f64, cp: f64, n_turbines: usize) -> f64 {
    let power_w = 0.5 * rho * ROTOR_AREA * cp * v.powi(3) * n_turbines as f64;
    let power_kw = power_w / 1000.0;

    let max_power = n_turbines as f64 * TURBINE_RATED_POWER_KW;
    power_kw.min(max_power)
}

/// Independent lognormal diesel price sampling (Appendix N/M.2).
///
/// Design 2 uses a lognormal distribution instead of GBM, encapsulating
/// the randomness without time-dependent price trends.
/// price = DIESEL_PRICE_INIT * exp((mu - 0.5*sigma^2) + sigma * z)
/// z ~ N(0, 1)
pub fn sample_diesel_price<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    let z: f64 = Normal::new(0.0, 1.0)
        .expect("valid standard normal")
        .sample(rng);

    DIESEL_PRICE_INIT * ((DIESEL_MU - 0.5 * DIESEL_SIGMA.powi(2)) + DIESEL_SIGMA * z).exp()
}

pub fn estimate_required_turbines(
    weather: &[WeatherRecord],
    throughput_tpd: f64,
    renewable_ratio: f64,
    rho: f64,
    cp: f64,
) -> usize {
    let demand_kwh = throughput_tpd * ENERGY_INTENSITY / 24.0;
    let target_kwh = renewable_ratio * demand_kwh;

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
/// Wind speed is drawn uniformly from the dataset (Appendix N) — no additional
/// perturbation. Diesel price is independently sampled from a lognormal distribution
/// each timestep (Appendix N/M.2). Cp and rho are sampled from their respective
/// normal distributions (Appendix N).
pub fn run_single_simulation<R: Rng + ?Sized>(
    rng: &mut R,
    weather: &[WeatherRecord],
    throughput_tpd: f64,
    renewable_ratio: f64,
) -> TrialOutput {
    let mut total_cost = 0.0;
    let mut total_emissions = 0.0;
    let mut total_diesel_kwh = 0.0;
    let mut total_demand_kwh = 0.0;

    // Appendix N: Cp ~ N(0.2901, 0.185^2)
    let cp = Normal::new(CP_MEAN, CP_STD)
        .expect("valid cp distribution")
        .sample(rng)
        .max(0.0);

    // Appendix N: rho ~ N(1.33, 0.08^2)
    let rho = Normal::new(RHO_MEAN, RHO_STD)
        .expect("valid rho distribution")
        .sample(rng)
        .max(0.0);

    let n_turbines = estimate_required_turbines(weather, throughput_tpd, renewable_ratio, rho, cp);

    let demand_kwh = throughput_tpd * ENERGY_INTENSITY / 24.0;
    let renewable_target = renewable_ratio * demand_kwh;
    let tonnes_per_hr = throughput_tpd / 24.0;

    let n_steps = weather.len();

    for _ in 0..n_steps {
        // Wind speed drawn uniformly from D (Appendix N)
        let idx = rng.random_range(0..weather.len());
        let row = &weather[idx];

        let Some(wind_speed) = row.wind_speed else {
            continue;
        };

        if !wind_speed.is_finite() {
            continue;
        }

        let wind_kwh = wind_power(wind_speed, rho, cp, n_turbines);
        let wind_used = wind_kwh.min(renewable_target);
        let diesel_needed = demand_kwh - wind_used;

        // Independent lognormal sample each timestep (Appendix N/M.2)
        let diesel_price = sample_diesel_price(rng);
        let diesel_liters = diesel_needed * FUEL_CONSUMPTION;
        let cost_diesel = diesel_liters * diesel_price;
        let cost_wind = wind_used * WIND_COST;

        // Emissions scale with diesel fraction of total demand
        let diesel_fraction = if demand_kwh > 0.0 { diesel_needed / demand_kwh } else { 0.0 };
        let emissions = tonnes_per_hr * EMISSION_INTENSITY * diesel_fraction;

        total_cost += cost_wind + cost_diesel;
        total_emissions += emissions;
        total_diesel_kwh += diesel_needed;
        total_demand_kwh += demand_kwh;
    }

    let years = n_steps as f64 / 8760.0;
    let cost_per_year = if years > 0.0 { total_cost / years } else { 0.0 };

    let avg_diesel_kw = if n_steps > 0 {
        total_diesel_kwh / n_steps as f64
    } else {
        0.0
    };

    let total_tonnes = (throughput_tpd / 24.0) * n_steps as f64;
    let emissions_intensity = if total_tonnes > 0.0 {
        total_emissions / total_tonnes
    } else {
        0.0
    };

    let wind_fraction = if total_demand_kwh > 0.0 {
        1.0 - (total_diesel_kwh / total_demand_kwh)
    } else {
        0.0
    };

    TrialOutput {
        emissions_intensity,
        cost_per_year,
        n_turbines,
        avg_diesel_kw,
        wind_fraction,
    }
}
