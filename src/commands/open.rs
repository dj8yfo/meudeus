
use crate::{
    database::{Database, SqliteAsyncHandle},
    dir::Directory,
    print::print_two_tokens,
    skim::open::Iteration, Open,
};

pub(crate) async fn exec(dir: Directory, db: SqliteAsyncHandle) -> Result<String, anyhow::Error> {
    dir.check()?;

    loop {
        let list = db.lock().await.list().await?;

        let multi = false;
        let note = Iteration::new(list, db.clone(), multi).run()?;
        note.open()?;


        println!("{}", print_two_tokens("viewed", &note.name()));
    }
}
