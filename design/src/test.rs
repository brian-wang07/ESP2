/// Deterministic single-turbine backtest.
///
/// Unlike the Monte Carlo, this walks weather records in chronological order
/// and uses fixed mean parameters (CP_MEAN, RHO_MEAN, DIESEL_PRICE_INIT) so
/// results are fully reproducible and comparable to historical data.
use crate::model::{
    wind_power, CP_MEAN, DIESEL_PRICE_INIT, EMISSION_INTENSITY, ENERGY_INTENSITY,
    FUEL_CONSUMPTION, RHO_MEAN, TURBINE_RATED_POWER_KW, WIND_COST,
};
use crate::types::WeatherRecord;

pub struct BacktestResult {
    /// Total wind energy generated over the period (kWh)
    pub total_wind_kwh: f64,
    /// Total electricity demand over the period (kWh)
    pub total_demand_kwh: f64,
    /// Fraction of demand met by the single turbine
    pub wind_fraction: f64,
    /// Capacity factor of the turbine (actual / rated)
    pub capacity_factor: f64,
    /// Total cost with one wind turbine (CAD)
    pub total_cost_wind: f64,
    /// Baseline cost if fully diesel (CAD)
    pub total_cost_diesel: f64,
    /// Cost saving vs all-diesel baseline (CAD)
    pub cost_saving: f64,
    /// Total emissions with wind (kg CO2)
    pub total_emissions_wind: f64,
    /// Baseline emissions at 0% renewables (kg CO2)
    pub total_emissions_diesel: f64,
    /// Emissions avoided vs all-diesel baseline (kg CO2)
    pub emissions_avoided: f64,
    /// Number of hourly records used
    pub n_steps: usize,
}

/// Run a single-turbine deterministic backtest over the full weather dataset.
///
/// Uses CP_MEAN, RHO_MEAN, and DIESEL_PRICE_INIT (no sampling).
/// Iterates weather records sequentially. Skips records with missing wind speed.
pub fn run_backtest(weather: &[WeatherRecord], throughput_tpd: f64) -> BacktestResult {
    let demand_kwh = throughput_tpd * ENERGY_INTENSITY / 24.0; // per hour
    let tonnes_per_hr = throughput_tpd / 24.0;

    let mut total_wind_kwh = 0.0;
    let mut total_demand_kwh = 0.0;
    let mut total_cost_wind = 0.0;
    let mut total_cost_diesel = 0.0_f64;
    let mut total_emissions_wind = 0.0;
    let mut total_emissions_diesel = 0.0_f64;
    let mut n_steps = 0usize;

    for record in weather {
        let Some(wind_speed) = record.wind_speed else {
            continue;
        };
        if !wind_speed.is_finite() {
            continue;
        }

        n_steps += 1;

        // Single turbine, deterministic coefficients
        let wind_kwh = wind_power(wind_speed, RHO_MEAN, CP_MEAN, 1);
        let wind_used = wind_kwh.min(demand_kwh);
        let diesel_needed = (demand_kwh - wind_used).max(0.0);

        let cost_wind_step = wind_used * WIND_COST
            + diesel_needed * FUEL_CONSUMPTION * DIESEL_PRICE_INIT;
        let cost_diesel_step = demand_kwh * FUEL_CONSUMPTION * DIESEL_PRICE_INIT;

        let diesel_fraction = if demand_kwh > 0.0 {
            diesel_needed / demand_kwh
        } else {
            0.0
        };
        let emissions_step = tonnes_per_hr * EMISSION_INTENSITY * diesel_fraction;
        let emissions_diesel_step = tonnes_per_hr * EMISSION_INTENSITY;

        total_wind_kwh += wind_kwh;
        total_demand_kwh += demand_kwh;
        total_cost_wind += cost_wind_step;
        total_cost_diesel += cost_diesel_step;
        total_emissions_wind += emissions_step;
        total_emissions_diesel += emissions_diesel_step;
    }

    let wind_fraction = if total_demand_kwh > 0.0 {
        (total_demand_kwh - (total_demand_kwh - total_wind_kwh.min(total_demand_kwh)))
            / total_demand_kwh
    } else {
        0.0
    };

    // Capacity factor: actual output / (rated power * hours)
    let capacity_factor = if n_steps > 0 && TURBINE_RATED_POWER_KW > 0.0 {
        total_wind_kwh / (TURBINE_RATED_POWER_KW * n_steps as f64)
    } else {
        0.0
    };

    BacktestResult {
        total_wind_kwh,
        total_demand_kwh,
        wind_fraction,
        capacity_factor,
        total_cost_wind,
        total_cost_diesel,
        cost_saving: total_cost_diesel - total_cost_wind,
        total_emissions_wind,
        total_emissions_diesel,
        emissions_avoided: total_emissions_diesel - total_emissions_wind,
        n_steps,
    }
}

pub fn print_backtest(r: &BacktestResult, throughput_tpd: f64) {
    let years = r.n_steps as f64 / 8760.0;
    println!("\n=== SINGLE-TURBINE BACKTEST ===");
    println!("Throughput:          {:.1} tpd", throughput_tpd);
    println!("Records used:        {} hours ({:.2} years)", r.n_steps, years);
    println!();
    println!("Turbine output:      {:.0} MWh total", r.total_wind_kwh / 1e3);
    println!("Demand:              {:.0} MWh total", r.total_demand_kwh / 1e3);
    println!("Wind fraction:       {:.1}%", r.wind_fraction * 100.0);
    println!("Capacity factor:     {:.1}%", r.capacity_factor * 100.0);
    println!();
    println!("Cost (wind+diesel):  CAD {:.2}M total  ({:.2}M/yr)",
        r.total_cost_wind / 1e6,
        r.total_cost_wind / years / 1e6);
    println!("Cost (all-diesel):   CAD {:.2}M total  ({:.2}M/yr)",
        r.total_cost_diesel / 1e6,
        r.total_cost_diesel / years / 1e6);
    println!("Cost saving:         CAD {:.2}M total  ({:.2}M/yr)",
        r.cost_saving / 1e6,
        r.cost_saving / years / 1e6);
    println!();
    println!("Emissions (wind):    {:.0} t CO2 total  ({:.0} t/yr)",
        r.total_emissions_wind / 1e3,
        r.total_emissions_wind / years / 1e3);
    println!("Emissions (diesel):  {:.0} t CO2 total  ({:.0} t/yr)",
        r.total_emissions_diesel / 1e3,
        r.total_emissions_diesel / years / 1e3);
    println!("Emissions avoided:   {:.0} t CO2 total  ({:.0} t/yr)",
        r.emissions_avoided / 1e3,
        r.emissions_avoided / years / 1e3);
}
