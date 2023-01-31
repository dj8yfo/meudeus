
use std::process::Command;

use crate::{
    database::{Database, SqliteAsyncHandle},
    dir::Directory,
    print::print_two_tokens,
    skim::explore::{Iteration, Action},
};

pub(crate) async fn exec(dir: Directory, db: SqliteAsyncHandle) -> Result<String, anyhow::Error> {
    dir.check()?;
    let mut list = db.lock().await.list().await?;

    loop {

        let out = Iteration::new(list.clone(), db.clone()).run().await?;

        match out.action {
            Action::Noop => {},
            Action::Open(note) => {
                if let Some(file_path) = note.file_path() {
                    Command::new("helix-22.12-x86_64.AppImage")
                        .arg(file_path.as_os_str())
                        .status()?;
                } else {
                    // for tags only list links

                }

                println!("{}", print_two_tokens("viewed", &note.name()));
                
            }
        }
        list = out.next_items;

    }
}
