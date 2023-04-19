use std::path::Path;
use toml;

use super::model::TrackConfig;

pub fn get_config_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<TrackConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let config: TrackConfig = toml::from_str(&contents)?;

    Ok(config)
}
