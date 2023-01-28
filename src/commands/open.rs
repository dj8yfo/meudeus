use std::process::Command;

use colored::Colorize;

use crate::{
    database::{Database, SqliteAsyncHandle},
    dir::Directory,
    print::print_two_tokens,
    skim::Search,
};

pub(crate) async fn exec(dir: Directory, db: SqliteAsyncHandle) -> Result<String, anyhow::Error> {
    dir.check()?;

    let list = db.lock().await.list().await?;

    let multi = false;
    let mut selections: Vec<_> = Search::new(list, db.clone(), multi)
        .run()?
        .into_iter()
        .take(1)
        .collect();

    let note = selections.remove(0);
    if let Some(file_path) = note.file_path() {
        Command::new("helix-22.12-x86_64.AppImage")
            .arg(file_path.as_os_str())
            .status()?;
    } else {
        // for tags only list links
    }

    println!("{}", print_two_tokens("viewed", &note.name()));
    Ok("success".cyan().to_string())
}
