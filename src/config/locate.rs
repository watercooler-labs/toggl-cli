use std::path::{Path, PathBuf};

use base64::{engine::general_purpose, Engine as _};

use crate::error::ConfigError;

pub fn locate_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_root = get_config_root();

    let mut config_path = std::env::current_dir()?;
    let mut config_filename = get_encoded_config_path(&config_root, &config_path);

    while !config_filename.exists() {
        if !config_path.pop() {
            return Err(Box::new(ConfigError::FileNotFound));
        }
        config_filename = get_encoded_config_path(&config_root, &config_path);
    }
    Ok(config_filename)
}

/// TODO: Cache config_path to avoid calling locate_tracked_path() multiple times
/// It's guaranteed to be the same for the duration of the program
pub fn locate_tracked_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_root = get_config_root();

    let mut config_path = std::env::current_dir()?;
    let mut config_filename = get_encoded_config_path(&config_root, &config_path);

    while !config_filename.exists() {
        if !config_path.pop() {
            return Err("No config file found".into());
        }
        config_filename = get_encoded_config_path(&config_root, &config_path);
    }
    Ok(config_path)
}

pub fn get_config_path_for_current_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_root = get_config_root();
    let path = std::env::current_dir()?;
    Ok(get_encoded_config_path(&config_root, &path))
}

fn get_config_root() -> PathBuf {
    directories::ProjectDirs::from("studio.watercooler", "labs", "toggl-cli")
        .unwrap()
        .config_dir()
        .to_path_buf()
}

fn get_encoded_config_path(config_root: &Path, path: &Path) -> PathBuf {
    let encoded = general_purpose::STANDARD.encode(
        path.to_str()
            .expect("Could not convert path to string")
            .as_bytes(),
    );
    config_root.join(format!("{}.toml", encoded))
}
