use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
    print::format_two_tokens,
    skim::open::Iteration,
};
use colored::Colorize;
use inquire::Text;
use syntect::easy::HighlightLines;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,

    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static, color_scheme).await?;

    let straight = true;
    let multi = false;
    let note = Iteration::new(
        "rename".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
        md_static,
        color_scheme,
        straight,
    )
    .run()
    .await?;

    rename(note, db, md_static).await?;

    Ok("success".truecolor(0, 255, 255).to_string())
}

pub(crate) async fn rename(
    mut note: Note,
    db: SqliteAsyncHandle,
    md_static: MarkdownStatic,
) -> Result<Note, anyhow::Error> {
    let new_name = Text::new("Enter new note's name:")
        .with_initial_value(&note.name())
        .prompt()?;

    let prev_name = note.name();
    db.lock().await.rename_note(&note, &new_name).await?;
    let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
    note.rename(&new_name, &mut highlighter, md_static);

    eprintln!(
        "{}",
        format_two_tokens("renamed", &format!("{} -> {}", prev_name, new_name))
    );
    Ok(note)
}
