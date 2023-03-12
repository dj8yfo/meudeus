use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    note::PreviewType,
    print::format_two_tokens,
    skim::explore::{Action, Iteration},
    Open,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    let mut preview_type = PreviewType::default();
    loop {
        let out = Iteration::new(
            list.clone(),
            db.clone(),
            external_commands.clone(),
            surf_parsing.clone(),
            preview_type,
        )
        .run()
        .await?;

        match out.action {
            Action::Noop => {}
            Action::Open(note) => {
                note.open(external_commands.open.clone())?;

                eprintln!("{}", format_two_tokens("viewed", &note.name()));
            }
            Action::TogglePreview => {
                preview_type = preview_type.toggle();
            }
        }
        list = out.next_items;
    }
}
