mod data;
mod types;
mod model;
mod config;
mod simulation;

use std::io;
use std::time::Instant;
use std::fs::File;

use config::SimulationConfig;
use simulation::{monte_carlo, summarize};

fn write_results_csv(
    path: &str,
    emissions: &[f64],
    costs: &[f64],
) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    writer.write_record(["emissions", "cost_per_kwh"])?;

    for (e, c) in emissions.iter().zip(costs.iter()) {
        writer.write_record([e.to_string(), c.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let weather = data::load_weather_csv("../weather/data.csv")?;

    let mut input = String::new();

    println!("Mine size scaling factor:");
    io::stdin().read_line(&mut input)?;
    let mine_scale: f64 = input.trim().parse()?;
    input.clear();

    println!("Renewable ratio (0-1):");
    io::stdin().read_line(&mut input)?;
    let renewable_ratio: f64 = input.trim().parse()?;

    let config = SimulationConfig::new(mine_scale, renewable_ratio, 100);


    let start = Instant::now();
    let results = monte_carlo(&weather, &config);

    let end = start.elapsed();
    let emissions_stats = summarize(&results.emissions);
    let cost_stats = summarize(&results.costs);

    println!("\n=== RESULTS ===");
    println!("Time Elapsed: {:?}", end);

    println!("Avg Emissions: count    {:.6e}", emissions_stats.count as f64);
    println!("mean     {:.6e}", emissions_stats.mean);
    println!("std      {:.6e}", emissions_stats.std_dev);
    println!("min      {:.6e}", emissions_stats.min);
    println!("25%      {:.6e}", emissions_stats.p25);
    println!("50%      {:.6e}", emissions_stats.p50);
    println!("75%      {:.6e}", emissions_stats.p75);
    println!("max      {:.6e}", emissions_stats.max);

    println!("Avg Cost per kWh: count    {:.6e}", cost_stats.count as f64);
    println!("mean     {:.6}", cost_stats.mean);
    println!("std      {:.6}", cost_stats.std_dev);
    println!("min      {:.6}", cost_stats.min);
    println!("25%      {:.6}", cost_stats.p25);
    println!("50%      {:.6}", cost_stats.p50);
    println!("75%      {:.6}", cost_stats.p75);
    println!("max      {:.6}", cost_stats.max);


    write_results_csv("monte_carlo_results.csv", &results.emissions, &results.costs)?;
    println!("Saved results to monte_carlo_results.csv");
    Ok(())
}