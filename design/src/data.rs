use std::path::Path;

use anyhow::{Context, Result};
use csv::ReaderBuilder;
use serde::Deserialize;

use crate::types::WeatherRecord;

#[derive(Debug, Deserialize)]
struct RawWeatherRecord {
    #[serde(rename = "WIND_SPEED")]
    wind_speed: Option<f64>,

}


/// Load weather data from CSV, matching the Python version's behavior:
/// - reads the CSV file
/// - keeps WIND_SPEED
/// - if LOCAL_MONTH exists, derives season
/// - does not do extra preprocessing
pub fn load_weather_csv<P: AsRef<Path>>(path: P) -> Result<Vec<WeatherRecord>> {
    let path_ref = path.as_ref();

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path_ref)
        .with_context(|| format!("failed to open weather CSV: {}", path_ref.display()))?;

    let mut records = Vec::new();

    for (i, row) in reader.deserialize::<RawWeatherRecord>().enumerate() {
        let raw = row.with_context(|| {
            format!(
                "failed to deserialize row {} in {}",
                i + 2,
                path_ref.display()
            )
        })?;


        records.push(WeatherRecord {
            wind_speed: raw.wind_speed,
        });
    }

    Ok(records)
}