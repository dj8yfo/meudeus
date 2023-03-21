use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    link::Link,
    note::PreviewType,
    print::format_two_tokens,
    skim::explore::{Action, Iteration},
    skim::surf::Iteration as SurfIteration,
    Open,
};
use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    surf: SurfParsing,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    let mut preview_type = PreviewType::default();
    let note = loop {
        let out = Iteration::new(
            list.clone(),
            db.clone(),
            external_commands.clone(),
            surf.clone(),
            preview_type,
        )
        .run()
        .await?;

        match out.action {
            Action::Noop => {}
            Action::Open(note) => {
                break note;
            }
            Action::TogglePreview => {
                preview_type = preview_type.toggle();
            }
        }
        list = out.next_items;
    };

    let all_vec = note.reachable_notes(db.clone()).await?;
    let links: std::io::Result<Vec<_>> = all_vec
        .into_iter()
        .map(|v| Link::parse(&v, &surf, &external_commands))
        .collect();
    let links: Vec<_> = links?.into_iter().flat_map(|v| v).collect();

    let link = SurfIteration::new(links, false).run()?;
    link.open(external_commands.open)?;

    eprintln!("{}", link);
    eprintln!("{}", format_two_tokens("surfed", &note.name()));

    Ok("success".cyan().to_string())
}
