// use colored::Colorize;
use crate::{config::parser, models::ResultWithDefaultError};

pub struct ConfigGetCommand;

impl ConfigGetCommand {
    pub async fn execute() -> ResultWithDefaultError<()> {
        println!(
            "{:?}",
            parser::get_config_from_file("./src/config/fixtures/base.toml")?
        );
        Ok(())
    }
}
