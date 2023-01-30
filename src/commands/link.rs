use colored::Colorize;

use crate::{
    database::{Database, SqliteAsyncHandle},
    dir::Directory,
    note::Note,
    print::print_two_tokens,
    skim::open::Iteration,
};

pub(crate) async fn exec(dir: Directory, db: SqliteAsyncHandle) -> Result<String, anyhow::Error> {
    dir.check()?;

    let list = db.lock().await.list().await?;

    let multi = false;
    let (mut from, mut to): (Option<Note>, Option<Note>) = (None, None);
    for target in [&mut from, &mut to] {
        let note = Iteration::new(list.clone(), db.clone(), multi).run()?;

        *target = Some(note);
    }

    let (from_key, to_key) = (from.as_ref().unwrap(), to.as_ref().unwrap());
    db.lock()
        .await
        .insert_link(&from_key.name(), &to_key.name())
        .await?;
    println!(
        "{}",
        print_two_tokens(
            "linked: ",
            &format!("\"{}\" -> \"{}\"", from.unwrap().name(), to.unwrap().name())
        )
    );

    Ok("success".cyan().to_string())
}
