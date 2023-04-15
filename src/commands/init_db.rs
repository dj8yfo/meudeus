use std::path::PathBuf;

use colored::Colorize;

use crate::database::Sqlite;

pub(crate) async fn exec(dir: PathBuf) -> Result<String, anyhow::Error> {
    Sqlite::new(true, dir).await?;
    Ok("initialized db".truecolor(0, 255, 255).to_string())
}
