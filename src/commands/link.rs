use colored::Colorize;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
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
        "link from".to_string(),
        list.clone(),
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
    )
    .run()
    .await?;

    link(from, db, &external_commands, &surf_parsing, md_static).await?;

    Ok("success".cyan().to_string())
}

pub(crate) async fn link(
    from: Note,
    db: SqliteAsyncHandle,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    md_static: MarkdownStatic,
) -> Result<(), anyhow::Error> {
    let name: String = from.name().chars().take(40).collect();

    let hint = format!("link from {}", name);
    let list = db.lock().await.list(md_static).await?;
    let to = Iteration::new(
        hint,
        list,
        db.clone(),
        false,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
    )
    .run()
    .await?;

    link_noninteractive(from, to, db).await?;
    Ok(())
}

pub(crate) async fn link_noninteractive(from: Note, to: Note, db: SqliteAsyncHandle) -> Result<(), anyhow::Error> {
    db.lock()
        .await
        .insert_link(&from.name(), &to.name())
        .await?;
    eprintln!(
        "{}",
        format_two_tokens(
            "linked: ",
            &format!("\"{}\" -> \"{}\"", from.name(), to.name())
        )
    );
    Ok(())
    
}
