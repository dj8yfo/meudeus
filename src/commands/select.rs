use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;
    let multi = false;
    let note = crate::skim::open::Iteration::new(
        "select".to_string(),
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
        md_static,
    )
    .run()
    .await?;

    Ok(note.name())
}
