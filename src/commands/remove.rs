use std::fs;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    print::format_two_tokens,
    skim::open::Iteration,
};
use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;

    let multi = false;
    let note = Iteration::new(
        "remove".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
        md_static,
    )
    .run()
    .await?;

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
