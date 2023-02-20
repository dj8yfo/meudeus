use crate::config::Config;
use colored::Colorize;

pub(crate) fn exec(config: Config) -> Result<String, anyhow::Error> {
    println!("{}", format!("{:#?}", config).yellow().to_string());

    Ok("success".cyan().to_string())
}
