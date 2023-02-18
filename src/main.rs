#[macro_use]
extern crate sql_builder;

#[macro_use]
extern crate lazy_static;

use clap::ArgMatches;

use colored::Colorize;
use std::{path::PathBuf, process::{exit, ExitStatus}, io};

mod commands;
mod database;
mod dir;
mod note;
mod link;
mod print;
mod skim;
mod external_commands;
pub(crate) use dir::Directory;

pub(crate) use database::Sqlite;

trait Open {
    fn open(&self) -> io::Result<ExitStatus>;
    
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let cmd = clap::Command::new("mds")
        .arg(clap::arg!(-d --"notes-dir" <NOTE_NAME>).value_parser(clap::value_parser!(PathBuf)))
        .bin_name("mds")
        .subcommand_required(true)
        .subcommand(
            clap::command!("n").arg(
                clap::arg!([title])
                    .value_parser(clap::value_parser!(String))
                    .required(true),
            ),
        )
        .subcommand(
            clap::command!("t").arg(
                clap::arg!([title])
                    .value_parser(clap::value_parser!(String))
                    .required(true),
            ),
        )
        .subcommand(clap::command!("i"))
        .subcommand(clap::command!("o"))
        .subcommand(clap::command!("e"))
        .subcommand(clap::command!("l"))
        .subcommand(clap::command!("s"));
    let matches = cmd.get_matches();

    let result = body(&matches).await;
    match result {
        Ok(print) => println!("{}", print),
        Err(err) => {
            println!("{}", format!("{:?}", err).red());
            exit(121)
        }
    }
}

async fn body(matches: &ArgMatches) -> anyhow::Result<String> {
    let dir = dir::Directory::new(matches.get_one::<PathBuf>("notes-dir"));
    let db_dir = dir.path.join(".sqlite");
    let result = match matches.subcommand() {
        Some(("i", _matches)) => commands::init_db::exec(db_dir).await,
        Some((subcommand, matches)) => {
            let db = match Sqlite::new(false, db_dir).await {
                Ok(db) => db,
                Err(err) => return Err(err.into()),
            };
            match subcommand {
                cmd @ "n" | cmd @ "t" => {
                    let is_tag = cmd == "t";
                    let title = matches
                        .get_one::<String>("title")
                        .ok_or(anyhow::anyhow!("empty title"))?;

                    commands::create::exec(title, dir, db, is_tag).await
                }
                "o" => commands::open::exec(dir, db).await,
                "e" => commands::explore::exec(dir, db).await,
                "l" => commands::link::exec(dir, db).await,
                "s" => commands::surf::exec(dir, db).await,
                _ => unreachable!("clap should ensure we don't get here"),
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
    result
}
