use crate::{
    config::ExternalCommands,
    database::{Database, SqliteAsyncHandle},
    print::print_two_tokens,
    skim::explore::{Action, Iteration},
    Open,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    loop {
        let out = Iteration::new(list.clone(), db.clone(), external_commands.clone())
            .run()
            .await?;

        match out.action {
            Action::Noop => {}
            Action::Open(note) => {
                note.open(external_commands.open.clone())?;

                println!("{}", print_two_tokens("viewed", &note.name()));
            }
        }
        list = out.next_items;
    }
}
