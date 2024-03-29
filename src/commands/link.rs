use colored::Colorize;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, PreviewType},
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
    let nested_threshold = 1;

    let from = Iteration::new(
        "link from".to_string(),
        list.clone(),
        db.clone(),
        multi,
        PreviewType::Details,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
        color_scheme,
        straight,
        nested_threshold,
    )
    .run()
    .await?;

    link(
        from,
        db,
        &external_commands,
        &surf_parsing,
        md_static,
        color_scheme,
        straight,
        nested_threshold,
    )
    .await?;

    Ok("success".truecolor(0, 255, 255).to_string())
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn link(
    from: Note,
    db: SqliteAsyncHandle,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    straight: bool,
    nested_threshold: usize,
) -> Result<(), anyhow::Error> {
    let name: String = from.name().chars().take(40).collect();

    let hint = format!("link from {}", name);
    let list = db.lock().await.list(md_static, color_scheme).await?;
    let to = Iteration::new(
        hint,
        list,
        db.clone(),
        false,
        PreviewType::Details,
        external_commands.clone(),
        surf_parsing.clone(),
        md_static,
        color_scheme,
        straight,
        nested_threshold,
    )
    .run()
    .await?;

    link_noninteractive(from, to, db, straight).await?;
    Ok(())
}

pub(crate) async fn link_noninteractive(
    from: Note,
    to: Note,
    db: SqliteAsyncHandle,
    straight: bool,
) -> Result<(), anyhow::Error> {
    db.lock()
        .await
        .insert_link(&from.name(), &to.name(), straight)
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
