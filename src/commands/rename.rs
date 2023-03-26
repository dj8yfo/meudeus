use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    print::format_two_tokens,
    skim::open::Iteration,
};
use colored::Colorize;
use inquire::Text;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list().await?;

    let multi = false;
    let note = Iteration::new(
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
    )
    .run().await?;

    let new_name = Text::new("Enter new note's name:")
        .with_initial_value(&note.name())
        .prompt()?;

    db.lock().await.rename_note(&note, &new_name).await?;

    eprintln!(
        "{}",
        format_two_tokens("renamed", &format!("{} -> {}", note.name(), new_name))
    );
    Ok("success".cyan().to_string())
}
