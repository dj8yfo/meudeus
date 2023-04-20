use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
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
    let note = crate::skim::open::Iteration::new(
        "select".to_string(),
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

    Ok(note.name())
}
