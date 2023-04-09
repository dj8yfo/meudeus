use colored::Colorize;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
    print::format_two_tokens,
    skim::open::Iteration,
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let list = db.lock().await.list(md_static).await?;

    let multi = false;
    let (mut from, mut to): (Option<Note>, Option<Note>) = (None, None);
    let (hint_from, hint_to) = ("link from".to_string(), "link to".to_string());
    for (target, hint) in [(&mut from, hint_from), (&mut to, hint_to)] {
        let note = Iteration::new(
            hint,
            list.clone(),
            db.clone(),
            multi,
            external_commands.clone(),
            surf_parsing.clone(),
            md_static,
        )
        .run()
        .await?;

        *target = Some(note);
    }

    let (from_key, to_key) = (from.as_ref().unwrap(), to.as_ref().unwrap());
    db.lock()
        .await
        .insert_link(&from_key.name(), &to_key.name())
        .await?;
    eprintln!(
        "{}",
        format_two_tokens(
            "linked: ",
            &format!("\"{}\" -> \"{}\"", from.unwrap().name(), to.unwrap().name())
        )
    );

    Ok("success".cyan().to_string())
}
