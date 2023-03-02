use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    print::format_two_tokens,
    skim::open::Iteration,
    skim::surf::Iteration as SurfIteration,
    Open,
};
use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    surf: SurfParsing,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list().await?;

    let multi = false;
    let note = Iteration::new(list, db.clone(), multi, external_commands.clone()).run()?;

    let all_vec = note.reachable_notes(db.clone()).await?;
    let links: std::io::Result<Vec<_>> = all_vec
        .into_iter()
        .map(|v| v.parse(&surf, &external_commands))
        .collect();
    let links: Vec<_> = links?.into_iter().flat_map(|v| v).collect();

    let link = SurfIteration::new(links, false).run()?;
    link.open(external_commands.open)?;

    eprintln!("{}", link);
    eprintln!("{}", format_two_tokens("surfed", &note.name()));

    Ok("success".cyan().to_string())
}
