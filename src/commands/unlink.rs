use colored::Colorize;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
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
    color_scheme: ColorScheme,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static, color_scheme).await?;
    let straight = true;

    let multi = false;
    let from = Iteration::new(
        "unlink from".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
        color_scheme,
        straight,
    )
    .run()
    .await?;

    unlink(
        from,
        db,
        &external_commands,
        &surf_parsing,
        md_static,
        color_scheme,
        straight,
    )
    .await?;

    Ok("success".truecolor(0, 255, 255).to_string())
}

pub(crate) async fn unlink(
    from: Note,
    db: SqliteAsyncHandle,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    straight: bool,
) -> Result<(), anyhow::Error> {
    let name: String = from.name().chars().take(40).collect();

    let hint = format!("unlink from {}", name);

    let forward_links = db
        .lock()
        .await
        .find_links_from(&from.name(), md_static, color_scheme, straight)
        .await?;
    let to = Iteration::new(
        hint,
        forward_links,
        db.clone(),
        false,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
        color_scheme,
        straight,
    )
    .run()
    .await?;

    db.lock()
        .await
        .remove_link(&from.name(), &to.name(), straight)
        .await?;
    eprintln!(
        "{}",
        format_two_tokens(
            "unlinked: ",
            &format!("\"{}\" -> \"{}\"", from.name(), to.name())
        )
    );
    Ok(())
}
