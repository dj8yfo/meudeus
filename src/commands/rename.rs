use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    note::Note,
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
        "rename".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
    )
    .run()
    .await?;

    rename(note, db).await?;

    Ok("success".cyan().to_string())
}

pub(crate) async fn rename(mut note: Note, db: SqliteAsyncHandle) -> Result<Note, anyhow::Error> {
    let new_name = Text::new("Enter new note's name:")
        .with_initial_value(&note.name())
        .prompt()?;

    let prev_name = note.name();
    db.lock().await.rename_note(&note, &new_name).await?;
    note.rename(&new_name);

    eprintln!(
        "{}",
        format_two_tokens("renamed", &format!("{} -> {}", prev_name, new_name))
    );
    Ok(note)
}
