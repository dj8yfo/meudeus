
use colored::Colorize;

use crate::{
    config::ExternalCommands,
    database::{Database, SqliteAsyncHandle},
    print::print_two_tokens,
    skim::open::Iteration,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list().await?;

    let multi = false;
    let from =
        Iteration::new(list, db.clone(), multi, external_commands.clone()).run()?;

    let forward_links = db.lock().await.find_links_from(&from.name()).await?;
    let to =
        Iteration::new(forward_links, db.clone(), multi, external_commands.clone()).run()?;

    db.lock()
        .await
        .remove_link(&from.name(), &to.name())
        .await?;
    println!(
        "{}",
        print_two_tokens(
            "unlinked: ",
            &format!("\"{}\" -> \"{}\"", from.name(), to.name())
        )
    );

    Ok("success".cyan().to_string())
}