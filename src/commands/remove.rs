use std::fs;

use crate::{
    config::ExternalCommands,
    database::{Database, SqliteAsyncHandle},
    print::format_two_tokens,
    skim::open::Iteration,
};
use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list().await?;

    let multi = false;
    let note = Iteration::new(list, db.clone(), multi, external_commands.clone()).run()?;

    db.lock().await.remove_note(&note).await?;

    if let Some(file_path) = note.file_path() {
        fs::remove_file(file_path)?;
    }
    eprintln!(
        "{}",
        format_two_tokens(
            "removed ",
            &format!("{}, {:?}", note.name(), note.file_path())
        )
    );
    Ok("success".cyan().to_string())
}
