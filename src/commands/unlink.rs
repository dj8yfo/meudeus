use colored::Colorize;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    print::format_two_tokens,
    skim::open::Iteration,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,

    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;

    let multi = false;
    let from = Iteration::new(
        "unlink from".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
    )
    .run()
    .await?;

    let forward_links = db
        .lock()
        .await
        .find_links_from(&from.name(), md_static)
        .await?;
    let to = Iteration::new(
        "unlink".to_string(),
        forward_links,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
        md_static,
    )
    .run()
    .await?;

    db.lock()
        .await
        .remove_link(&from.name(), &to.name())
        .await?;
    eprintln!(
        "{}",
        format_two_tokens(
            "unlinked: ",
            &format!("\"{}\" -> \"{}\"", from.name(), to.name())
        )
    );

    Ok("success".cyan().to_string())
}
