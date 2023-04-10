use std::{collections::HashMap, path::Path};
use toml;

use super::model::{BranchConfig, TrackConfig};

const DEFAULT_BRANCH: &str = "*";

pub fn get_config_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<TrackConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let mut branches: HashMap<String, BranchConfig> = toml::from_str(&contents)?;

    let default_config = branches.get(DEFAULT_BRANCH).cloned().unwrap_or_default();

    // Delete the default config from the branches config
    branches.remove(DEFAULT_BRANCH);

    Ok(TrackConfig {
        default: default_config,
        branches,
    })
}
