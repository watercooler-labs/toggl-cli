use crate::models::ResultWithDefaultError;

pub struct ConfigGetCommand;

impl ConfigGetCommand {
    pub async fn execute() -> ResultWithDefaultError<()> {
        println!(
            "{:?}",
            super::locate::locate_config()?
        );
        Ok(())
    }
}
