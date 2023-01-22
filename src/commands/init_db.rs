use std::path::PathBuf;

use crate::database::Sqlite;

pub(crate) async fn exec(dir: PathBuf) -> Result<String, anyhow::Error> {
    Sqlite::new(true, dir).await?;
    Ok("Initialized DB".to_string())
}
