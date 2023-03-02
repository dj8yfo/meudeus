
use crate::{
    config::ExternalCommands,
    database::{Database, SqliteAsyncHandle},
};


pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
        let list = db.lock().await.list().await?;
        let multi = false;
        let note = crate::skim::open::Iteration::new(list, db.clone(), multi, external_commands.clone()) .run()? ;

    Ok(note.name())
}
