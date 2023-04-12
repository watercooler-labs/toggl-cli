use std::path::{Path, PathBuf};

use base64::{engine::general_purpose, Engine as _};

use super::model::TrackConfig;

pub fn locate_config() -> Result<TrackConfig, Box<dyn std::error::Error>> {
    let config_filename = locate_config_path()?;
    super::parser::get_config_from_file(config_filename)
}

pub fn locate_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_root = get_config_root();
    let mut config_path = std::env::current_dir()?;

    let mut config_filename = get_encoded_config_path(&config_root, &config_path);
    while !config_filename.exists() {
        if !config_path.pop() {
            let global_config_filename = config_root.join("global.toml");
            if global_config_filename.exists() {
                config_filename = global_config_filename;
                break;
            }
            return Err("No config file found".into());
        }
        config_filename = get_encoded_config_path(&config_root, &config_path);
    }
    Ok(config_filename)
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
