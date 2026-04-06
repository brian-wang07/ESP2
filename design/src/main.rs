mod data;
mod types;
mod model;
mod config;
mod simulation;
mod test;

use std::io;
use std::time::Instant;
use std::fs::File;

use config::SimulationConfig;
use simulation::{monte_carlo, summarize};

fn write_results_csv(
    path: &str,
    emissions: &[f64],
    costs: &[f64],
    social_acceptance: &[f64],
) -> anyhow::Result<()> {
    let file = File::create(path)?;
    let mut writer = csv::Writer::from_writer(file);

    writer.write_record(["emissions", "cost_per_year", "social_acceptance"])?;

    for ((e, c), sa) in emissions.iter().zip(costs.iter()).zip(social_acceptance.iter()) {
        writer.write_record([e.to_string(), c.to_string(), sa.to_string()])?;
    }

    writer.flush()?;
    Ok(())
}

/// Read peak virtual memory size (VmPeak) and resident set size (VmRSS) from
/// /proc/self/status. Returns (vm_peak_kb, vm_rss_kb).
fn read_proc_memory() -> (u64, u64) {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return (0, 0);
    };
    let mut vm_peak = 0u64;
    let mut vm_rss = 0u64;
    for line in status.lines() {
        if line.starts_with("VmPeak:") {
            vm_peak = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if line.starts_with("VmRSS:") {
            vm_rss = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        }
    }
    (vm_peak, vm_rss)
}

fn main() -> anyhow::Result<()> {
    let weather = data::load_weather_csv("../weather/data.csv")?;

    let mut input = String::new();

    println!("Mine throughput (tpd):");
    io::stdin().read_line(&mut input)?;
    let throughput_tpd: f64 = input.trim().parse()?;
    input.clear();

    println!("Renewable ratio (0-1):");
    io::stdin().read_line(&mut input)?;
    let renewable_ratio: f64 = input.trim().parse()?;
    input.clear();

    println!("Affected population:");
    io::stdin().read_line(&mut input)?;
    let population: f64 = input.trim().parse()?;
    input.clear();

    println!("Number of iterations:");
    io::stdin().read_line(&mut input)?;
    let n_trials: usize = input.trim().parse()?;

    let config = SimulationConfig::new(throughput_tpd, renewable_ratio, n_trials);

    let start = Instant::now();
    let results = monte_carlo(&weather, &config, population);
    let elapsed = start.elapsed();

    let (vm_peak_kb, vm_rss_kb) = read_proc_memory();

    let emissions_stats = summarize(&results.emissions);
    let cost_stats = summarize(&results.costs);
    let sa_stats = summarize(&results.social_acceptance);

    println!("\n=== RESULTS ===");
    println!("Time elapsed: {:?}", elapsed);
    println!("Peak virtual memory: {:.1} KB ({:.2} MB)", vm_peak_kb as f64, vm_peak_kb as f64 / 1024.0);
    println!("Resident set size:   {:.1} KB ({:.2} MB)", vm_rss_kb as f64, vm_rss_kb as f64 / 1024.0);

    println!("\nEmissions Intensity (kg CO2/t):");
    println!("  count  {:.6e}", emissions_stats.count as f64);
    println!("  mean   {:.6e}", emissions_stats.mean);
    println!("  std    {:.6e}", emissions_stats.std_dev);
    println!("  min    {:.6e}", emissions_stats.min);
    println!("  25%    {:.6e}", emissions_stats.p25);
    println!("  50%    {:.6e}", emissions_stats.p50);
    println!("  75%    {:.6e}", emissions_stats.p75);
    println!("  max    {:.6e}", emissions_stats.max);

    println!("\nAnnual Cost (CAD/year):");
    println!("  count  {:.6e}", cost_stats.count as f64);
    println!("  mean   {:.6e}", cost_stats.mean);
    println!("  std    {:.6e}", cost_stats.std_dev);
    println!("  min    {:.6e}", cost_stats.min);
    println!("  25%    {:.6e}", cost_stats.p25);
    println!("  50%    {:.6e}", cost_stats.p50);
    println!("  75%    {:.6e}", cost_stats.p75);
    println!("  max    {:.6e}", cost_stats.max);

    println!("\nSocial Acceptance Score:");
    println!("  count  {:.6e}", sa_stats.count as f64);
    println!("  mean   {:.6}", sa_stats.mean);
    println!("  std    {:.6}", sa_stats.std_dev);
    println!("  min    {:.6}", sa_stats.min);
    println!("  25%    {:.6}", sa_stats.p25);
    println!("  50%    {:.6}", sa_stats.p50);
    println!("  75%    {:.6}", sa_stats.p75);
    println!("  max    {:.6}", sa_stats.max);

    write_results_csv(
        "monte_carlo_results.csv",
        &results.emissions,
        &results.costs,
        &results.social_acceptance,
    )?;
    println!("\nSaved results to monte_carlo_results.csv");

    //let backtest = test::run_backtest(&weather, throughput_tpd);
    //test::print_backtest(&backtest, throughput_tpd);

    Ok(())
}
