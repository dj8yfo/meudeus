use crate::{config::Config, print::format_two_tokens};
use colored::Colorize;

pub(crate) fn exec(config: Config) -> Result<String, anyhow::Error> {
    let prefix = format!("{:#?}", config).truecolor(255, 255, 0).to_string();
    let suffix = format_two_tokens("config", "valid");
    Ok(format!("{}\n\n{}", prefix, suffix))
}
