use colored::Colorize;
use crate::{
    database::{Database, SqliteAsyncHandle},
    dir::Directory,
    print::print_two_tokens,
    skim::open::Iteration,
    skim::surf::Iteration as SurfIteration, Open,
};

pub(crate) async fn exec(dir: Directory, db: SqliteAsyncHandle) -> Result<String, anyhow::Error> {
    dir.check()?;

    let list = db.lock().await.list().await?;

    let multi = false;
    let note = Iteration::new(list, db.clone(), multi).run()?;

    let all_vec = note.reachable_notes(db.clone()).await?;
    let links: std::io::Result<Vec<_>> = all_vec.into_iter().map(|v| v.parse()).collect();
    let links: Vec<_> = links?.into_iter().flat_map(|v| v).collect();


    let link = SurfIteration::new(links, false).run()?;
    link.open()?;


    println!("{}", print_two_tokens("opened", &format!("{}", link)));
    println!("{}", print_two_tokens("surfed", &note.name()));

    Ok("success".cyan().to_string())
}
