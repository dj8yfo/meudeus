use crate::{
    commands::link::link,
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, PreviewType},
    print::format_two_tokens,
    skim::explore::{Action, Iteration},
    Open,
};

use super::{remove::remove, rename::rename, unlink::unlink};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list(md_static).await?;

    let mut preview_type = PreviewType::default();
    loop {
        let (next_items, opened, preview_type_after) = iteration(
            db.clone(),
            list,
            &external_commands,
            &surf_parsing,
            preview_type,
            md_static,
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
                let note = rename(opened, db.clone(), md_static).await?;
                list = vec![note];
            }

            Some(Action::Link(linked_from)) => {
                if let Err(err) = link(
                    linked_from.clone(),
                    db.clone(),
                    &external_commands,
                    &surf_parsing,
                    md_static,
                )
                .await
                {
                    eprintln!("link error: {:?}", err);
                }
                list = vec![linked_from];
            }

            Some(Action::Unlink(unlinked_from)) => {
                if let Err(err) = unlink(
                    unlinked_from.clone(),
                    db.clone(),
                    &external_commands,
                    &surf_parsing,
                    md_static,
                )
                .await
                {
                    eprintln!("unlink error: {:?}", err);
                }
                list = vec![unlinked_from];
            }

            Some(Action::Remove(removed)) => {
                let next = match remove(db.clone(), removed.clone(), true).await {
                    Ok(true) => vec![],
                    Ok(false) => vec![removed],
                    Err(err) => {
                        eprintln!("remove error: {:?}", err);
                        vec![removed]
                    }
                };
                list = next;
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
    md_static: MarkdownStatic,
) -> Result<(Vec<Note>, Option<Action>, PreviewType), anyhow::Error> {
    let out = Iteration::new(
        list.clone(),
        db.clone(),
        external_commands.clone(),
        surf_parsing.clone(),
        preview_type,
        md_static,
    )
    .run()
    .await?;

    let res = match out.action {
        Action::Back | Action::Forward => (out.next_items, None, preview_type),
        Action::Widen => (db.lock().await.list(md_static).await?, None, preview_type),
        action @ Action::Open(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Rename(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Link(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Unlink(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Remove(..) => (out.next_items, Some(action), preview_type),
        Action::TogglePreview => (out.next_items, None, preview_type.toggle()),
    };
    Ok(res)
}
