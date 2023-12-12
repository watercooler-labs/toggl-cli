use crate::{models::ResultWithDefaultError, utilities};
use colored::Colorize;

pub struct ConfigInitCommand;

impl ConfigInitCommand {
    pub async fn execute(edit: bool) -> ResultWithDefaultError<()> {
        let config_path = super::locate::locate_config_path();
        match config_path {
            Ok(path) => {
                if edit {
                    utilities::open_path_in_editor(path)?;
                } else {
                    let display_path = utilities::simplify_config_path_for_display(path.as_path());
                    println!(
                        "{} {}",
                        "Config file already exists at".yellow(),
                        display_path
                    );
                    return Ok(());
                }
            }
            Err(_) => {
                let default_config = include_bytes!("./fixtures/default.toml");
                let config_path = super::locate::get_config_path_for_current_dir()?;
                let config_dir = config_path.parent().unwrap();
                let display_config_path = utilities::simplify_config_path_for_display(config_dir);
                let msg = format!(
                    "{} {}",
                    "Created config at".green().bold(),
                    display_config_path
                );
                std::fs::create_dir_all(config_dir).expect("failed to create config directory");
                std::fs::write(config_path, default_config).expect("failed to write config");
                println!("{}", msg);
            }
        }
        Ok(())
    }
}
