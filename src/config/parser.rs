use std::path::Path;
use toml;

use crate::models::ResultWithDefaultError;

use super::model::TrackConfig;

pub fn get_config_from_file<P: AsRef<Path>>(path: P) -> ResultWithDefaultError<TrackConfig> {
    let contents = std::fs::read_to_string(path).expect("failed to read config file");
    let config: TrackConfig = toml::from_str(&contents).expect("failed to parse config");

    Ok(config)
}
