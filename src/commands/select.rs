use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list().await?;
    let multi = false;
    let note = crate::skim::open::Iteration::new(
        list,
        db.clone(),
        multi,
        external_commands.clone(),
        surf_parsing,
    )
    .run().await?;

    Ok(note.name())
}
