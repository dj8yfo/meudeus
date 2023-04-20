use std::fs;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
    print::format_two_tokens,
    skim::open::Iteration,
};
use colored::Colorize;

use inquire::Confirm;

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
    let nested_threshold = 1;
    let note = Iteration::new(
        "remove".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
        md_static,
        color_scheme,
        straight,
        nested_threshold,
    )
    .run()
    .await?;
    remove(db, note, false).await?;

    Ok("success".truecolor(0, 255, 255).to_string())
}

pub(crate) async fn remove(
    db: SqliteAsyncHandle,
    note: Note,
    confirm: bool,
) -> Result<bool, anyhow::Error> {
    if confirm {
        let ans = Confirm::new(&format!("sure you want to delete `{}`", note.name())).prompt();
        match ans {
            Ok(true) => {}
            Ok(false) => return Ok(false),
            Err(err) => {
                return Err(err)?;
            }
        }
    }
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
    Ok(true)
}
