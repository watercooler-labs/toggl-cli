use crate::models::ResultWithDefaultError;

pub struct ConfigActiveCommand;

impl ConfigActiveCommand {
    pub async fn execute() -> ResultWithDefaultError<()> {
        let config_path = super::locate::locate_config_path()?;
        let track_config = super::parser::get_config_from_file(config_path)?;
        println!("{}", track_config.get_active_config()?);
        Ok(())
    }
}
