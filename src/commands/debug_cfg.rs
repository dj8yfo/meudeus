use crate::config::Config;
use colored::Colorize;

pub(crate) fn exec(config: Config) -> Result<String, anyhow::Error> {
    Ok(format!("{:#?}", config).yellow().to_string())
}
