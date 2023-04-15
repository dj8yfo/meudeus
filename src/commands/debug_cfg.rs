use crate::config::Config;
use colored::Colorize;

pub(crate) fn exec(config: Config) -> Result<String, anyhow::Error> {
    Ok(format!("{:#?}", config).truecolor(255, 255, 0).to_string())
}
