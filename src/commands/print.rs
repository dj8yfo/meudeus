use std::collections::HashSet;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    print::format_two_tokens,
};

use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    name: Option<String>,
) -> Result<String, anyhow::Error> {
    let note = {
        if let Some(name) = name {
            let note = db.lock().await.get(&name).await?;
            note
        } else {
            let list = db.lock().await.list().await?;
            let multi = false;
            crate::skim::open::Iteration::new(
                "print".to_string(),
                list,
                db.clone(),
                multi,
                external_commands.clone(),
                surf_parsing.clone(),
            )
            .run()
            .await?
        }
    };

    let (tree, _) = note
        .construct_link_term_tree(0, HashSet::new(), external_commands, surf_parsing, db)
        .await?;

    println!("{}", tree);

    eprintln!(
        "{}",
        format_two_tokens("printed", &format!("{}", note.name()))
    );
    Ok("success".cyan().to_string())
}
