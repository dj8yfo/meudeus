use clap::ArgMatches;
use colored::Colorize;
use std::{path::PathBuf, process::exit};

mod commands;
mod database;
mod dir;
mod note;
pub(crate) use dir::Directory;

pub(crate) use database::Sqlite;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmd = clap::Command::new("zett")
        .arg(clap::arg!(-d --"notes-dir" <NOTE_NAME>).value_parser(clap::value_parser!(PathBuf)))
        .bin_name("zett")
        .subcommand_required(true)
        .subcommand(
            clap::command!("create")
                .arg(
                    clap::arg!(-t --"title" <NOTE_NAME>)
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                )
                .arg(clap::arg!(--"tag")),
        )
        .subcommand(clap::command!("init_db"));
    let matches = cmd.get_matches();

    let result = body(&matches).await;
    match result {
        Ok(print) => println!("{}", print.green()),
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
        Some(("init_db", _matches)) => commands::init_db::exec(db_dir).await,
        Some((subcommand, matches)) => {
            let db = match Sqlite::new(false, db_dir).await {
                Ok(db) => db,
                Err(err) => return Err(err.into()),
            };
            match subcommand {
                "create" => commands::create::exec(dir, matches, db).await,
                _ => unreachable!("clap should ensure we don't get here"),
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
    result
}
