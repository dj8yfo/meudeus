use std::time::Duration;

use tokio::time::sleep;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    link::Link,
    note::PreviewType,
    print::format_two_tokens,
    skim::explore::Action,
    skim::surf::Action as SurfAction,
    skim::surf::Iteration as SurfIteration,
    Jump, Open,
};

use super::explore::iteration;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    surf: SurfParsing,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    let mut preview_type = PreviewType::default();
    let note = loop {
        let (next_items, opened, preview_type_after) = iteration(
            db.clone(),
            list,
            &external_commands,
            &surf,
            preview_type,
        )
        .await?;
        preview_type = preview_type_after;
        list = next_items;

        if let Some(Action::Open(opened)) = opened {
            break opened;
        }
    };

    loop {
        let all_vec = note.reachable_notes(db.clone()).await?;
        let links: std::io::Result<Vec<_>> = all_vec
            .into_iter()
            .map(|v| Link::parse(&v, &surf))
            .collect();
        let links: Vec<_> = links?.into_iter().flat_map(|v| v).collect();
        let action = SurfIteration::new(links, false, external_commands.clone())
            .run()
            .await?;
        eprintln!("{}", action);
        match action {
            SurfAction::Open(ref link) => {
                link.open(external_commands.clone().open)?;
                eprintln!("{}", link.preview_item.as_ref().unwrap());
            }
            SurfAction::Jump(ref link) => {
                link.jump(external_commands.clone().open)?;
                eprintln!("{}", link.preview_item.as_ref().unwrap());
            }
        }
        sleep(Duration::new(1, 500_000_000)).await;
        eprintln!("{}", format_two_tokens("surfed", &note.name()));
    }
}
