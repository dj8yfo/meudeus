use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    note::{Note, PreviewType},
    print::format_two_tokens,
    skim::explore::{Action, Iteration},
    Open,
};

use super::rename::rename;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    let mut preview_type = PreviewType::default();
    loop {
        let (next_items, opened, preview_type_after) = iteration(
            db.clone(),
            list,
            &external_commands,
            &surf_parsing,
            preview_type,
        )
        .await?;
        preview_type = preview_type_after;
        list = next_items;

        match opened {
            Some(Action::Open(opened)) => {
                opened.open(external_commands.open.clone())?;
                eprintln!("{}", format_two_tokens("viewed", &opened.name()));
            }
            Some(Action::Rename(opened)) => {
                let note = rename(opened, db.clone()).await?;
                list = vec![note];
            }
            _ => {}
        }
    }
}

pub async fn iteration(
    db: SqliteAsyncHandle,
    list: Vec<Note>,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    preview_type: PreviewType,
) -> Result<(Vec<Note>, Option<Action>, PreviewType), anyhow::Error> {
    let out = Iteration::new(
        list.clone(),
        db.clone(),
        external_commands.clone(),
        surf_parsing.clone(),
        preview_type,
    )
    .run()
    .await?;

    let res = match out.action {
        Action::Back | Action::Forward => (out.next_items, None, preview_type),
        Action::Widen => (db.lock().await.list().await?, None, preview_type),
        action @ Action::Open(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Rename(..) => (out.next_items, Some(action), preview_type),
        Action::TogglePreview => (out.next_items, None, preview_type.toggle()),
    };
    Ok(res)
}
