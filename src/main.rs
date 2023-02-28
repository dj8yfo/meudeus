#[macro_use]
extern crate sql_builder;

use clap::ArgMatches;

use colored::Colorize;
use std::{
    env, io,
    path::PathBuf,
    process::{exit, ExitStatus},
};
use config::Open as OpenCfg;

mod commands;
mod config;
mod database;
mod external_commands;
mod link;
mod note;
mod print;
mod skim;

pub(crate) use database::Sqlite;

trait Open {
    fn open(&self, cfg: OpenCfg) -> io::Result<Option<ExitStatus>>;
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    let cmd = clap::Command::new("mds")
        .version("v0.5.0")
        .bin_name("mds")
        .subcommand_required(true)
        .subcommand(clap::command!("debug-cfg").about("print `Debug` representtion of `config`"))
        .subcommand(
            clap::command!("init")
                .about("`initialize` .sqlite database in notes dir, specified by config"),
        )
        .subcommand(
            clap::command!("n").about("create a `note`").arg(
                clap::arg!([title] "note title (unique name among notes and tags)")
                    .value_parser(clap::value_parser!(String))
                    .required(true),
            ),
        )
        .subcommand(
            clap::command!("t")
                .about("create a `tag` (note without file body)")
                .arg(
                    clap::arg!([title] "tag title (unique name among notes and tags)")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
        .subcommand(
            clap::command!("l").about("`link` 2 notes A -> B, selected twice in skim interface"),
        )
        .subcommand(
            clap::command!("o").about("start an infinite skim selection loop to `open` notes/tags"),
        )
        .subcommand(
            clap::command!("e")
                .about("`explore` notes by <c-h> (backlinks) , <c-l> (links forward)"),
        )
        .subcommand(clap::command!("s").about(
        "`surf` (fuzzy find) through all `[markdown reference](links)` 
        and ```code_block(s)```, found in all notes, 
        reachable by forward links from note/tag S, 
        selected interactively by skim",
        ))
        .subcommand(
            clap::command!("unlink").about("`unlink` 2 notes A -> B, selected twice in skim interface"),
        )
        .subcommand(
            clap::command!("remove").about("`remove` note R, selected in skim interface"),
        )
        .subcommand(
            clap::command!("rename").about("`rename` note R, selected in skim interface"),
        )
    ;
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
    let config = config::Config::parse()?;

    if let Err(err) = env::set_current_dir(&config.work_dir) {
        println!(
            "{}",
            format!("couldn't change work dir to {:?}", &config.work_dir)
                .red()
                .to_string()
        );
        return Err(err)?;
    }

    let db_dir = PathBuf::from("./.sqlite");
    let result = match matches.subcommand() {
        Some(("init", _matches)) => commands::init_db::exec(db_dir).await,
        Some(("debug-cfg", _matches)) => commands::debug_cfg::exec(config),
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

                    commands::create::exec(title, db, is_tag).await
                }
                "o" => commands::open::exec(db, config.external_commands).await,
                "e" => commands::explore::exec(db, config.external_commands).await,
                "l" => commands::link::exec(db, config.external_commands).await,
                "s" => {
                    commands::surf::exec(db, config.surf_parsing, config.external_commands).await
                }
                "unlink" => commands::unlink::exec(db, config.external_commands).await,
                "remove" => commands::remove::exec(db, config.external_commands).await,
                "rename" => commands::rename::exec(db, config.external_commands).await,
                _ => unreachable!("clap should ensure we don't get here"),
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
    result
}
