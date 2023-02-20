use crate::{
    config::ExternalCommands,
    database::{Database, SqliteAsyncHandle},
    print::print_two_tokens,
    skim::open::Iteration,
    Open,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    loop {
        let list = db.lock().await.list().await?;

        let multi = false;
        let note = Iteration::new(list, db.clone(), multi, external_commands.clone()).run()?;
        note.open(external_commands.open.clone())?;

        println!("{}", print_two_tokens("viewed", &note.name()));
    }
}
