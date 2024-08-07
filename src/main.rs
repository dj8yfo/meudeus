#[macro_use]
extern crate sql_builder;

use clap::{Arg, ArgAction, ArgMatches};

use colored::Colorize;
use config::{color::Color, Open as OpenCfg};
use highlight::static_markdown_syntax;
use std::{
    env, io,
    path::PathBuf,
    process::{exit, ExitStatus},
};
use syntect::highlighting::{Theme, ThemeSet};

mod commands;
mod config;
mod database;
mod external_commands;
mod highlight;
mod lines;
mod link;
mod note;
mod print;
mod skim;
mod task_item;

pub(crate) use database::Sqlite;

trait Open {
    fn open(&self, cfg: OpenCfg) -> io::Result<Option<ExitStatus>>;

    fn open_xdg(&self) -> Result<(), opener::OpenError>;
}

trait Jump {
    fn jump(&self, cfg: OpenCfg) -> io::Result<Option<ExitStatus>>;
}

trait Yank {
    fn yank(&self, cfg: OpenCfg) -> io::Result<Option<ExitStatus>>;
}

fn load_theme(color: Color) -> Option<&'static Theme> {
    let theme = ThemeSet::get_theme(color.theme.0).ok();
    match theme {
        Some(theme) => {
            let boxed_theme = Box::new(theme);
            let static_theme: &'static Theme = Box::leak(boxed_theme);
            Some(static_theme)
        }
        None => None,
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() {
    env_logger::init();
    let cmd = clap::Command::new("mds")
        .version("v0.19.2")
        .about("meudeus v0.19.2\na skim shredder for plain-text papers")
        .bin_name("mds")
        .arg(clap::arg!(-c --color  "whether color output should be forced"))
        .subcommand_required(true)
        .subcommand(clap::command!("debug-cfg").about("print Debug representtion of config"))
        .subcommand(
            clap::command!("init")
                .about("`initialize` .sqlite database in notes dir, specified by config"),
        )
        .subcommand(
            clap::command!("note")
                .visible_alias("n")
                .about("create a note")
                .arg(
                    clap::arg!([title] "note title (unique name among notes and tags)")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
        .subcommand(
            clap::command!("tag")
                .visible_alias("t")
                .about("create a tag (note without file body)")
                .arg(
                    clap::arg!([title] "tag title (unique name among notes and tags)")
                        .value_parser(clap::value_parser!(String))
                        .required(true),
                ),
        )
        .subcommand(clap::command!("select").about("select note S, i.e. print its name to stdout"))
        .subcommand(
            clap::command!("link")
                .visible_alias("l")
                .about("link 2 notes A -> B, selected twice in skim interface"),
        )
        .subcommand(
            clap::command!("unlink")
                .visible_alias("ul")
                .about("unlink 2 notes A -> B, selected twice in skim interface"),
        )
        .subcommand(
            clap::command!("remove")
                .visible_alias("rm")
                .about("remove note R, selected in skim interface"),
        )
        .subcommand(
            clap::command!("rename")
                .visible_alias("mv")
                .about("rename note R, selected in skim interface"),
        )
        .subcommand(
            clap::command!("print")
                .visible_alias("p")
                .about("print subgraph of notes and links reachable downwards from selected note P")
                .arg(
                    clap::arg!(-n --name <NOTE_NAME> "note name")
                        .value_parser(clap::value_parser!(String))
                        .required(false),
                ),
        )
        .subcommand(
            clap::command!("explore")
                .arg(
                    Arg::new("select")
                        .short('s')
                        .long("select")
                        .action(ArgAction::Append)
                        .value_name("NOTE_NAME_FULL")
                        .help(
                            "full name of note to start explore with (may be used multiple times)",
                        ),
                )
                .visible_alias("ex")
                .about("explore notes by <c-h> (backlinks) , <c-l> (links forward)"),
        )
        .subcommand(
            clap::command!("surf").visible_alias("s").about(
                "surf through all links and code snippets found downwards from selected note S",
            ),
        )
        .subcommand(
            clap::command!("stack")
                .visible_alias("st")
                .about("browse GLOBAL stack of notes"),
        )
        .subcommand(clap::command!("checkmark").visible_alias("k").about(
            "checkmark, toggle state TODO/DONE of multiple task items, found in a selected note C",
        ));

    let matches = cmd.get_matches();
    if matches.get_flag("color") {
        colored::control::set_override(true);
    }

    let result = body(&matches).await;
    match result {
        Ok(print) => println!("{}", print),
        Err(err) => {
            eprintln!("{}", format!("{:?}", err).truecolor(255, 0, 0));
            exit(121)
        }
    }
}

async fn body(matches: &ArgMatches) -> anyhow::Result<String> {
    let config = match config::Config::parse() {
        Ok(config) => config,
        Err(err) => {
            println!("{:?}", err);
            return Err(anyhow::anyhow!("config error"));
        }
    };
    let loaded_theme = load_theme(config.color.clone());

    if let Err(err) = env::set_current_dir(&config.work_dir.0) {
        eprintln!(
            "{}",
            format!("couldn't change work dir to {:?}", &config.work_dir).red()
        );
        return Err(err)?;
    }

    let db_dir = PathBuf::from("./.sqlite");
    let md_static = static_markdown_syntax(loaded_theme);
    let surf_bindings = config.keymap.surf.clone().try_into()?;
    let checkmark_bindings = config.keymap.checkmark.clone().try_into()?;
    let stack_bindings = config.keymap.stack.clone().try_into()?;
    let explore_bindings = config.keymap.explore.clone().try_into()?;

    let result = match matches.subcommand() {
        Some(("init", _matches)) => commands::init_db::exec(db_dir).await,
        Some(("debug-cfg", _matches)) => commands::debug_cfg::exec(config),
        Some((subcommand, matches)) => {
            let db = match Sqlite::new(false, db_dir).await {
                Ok(db) => db,
                Err(err) => return Err(err.into()),
            };
            match subcommand {
                cmd @ "note" | cmd @ "tag" => {
                    let is_tag = cmd == "tag";
                    let title = matches
                        .get_one::<String>("title")
                        .ok_or(anyhow::anyhow!("empty title"))?;

                    commands::create::exec(title, db, is_tag, md_static, config.color.elements)
                        .await
                }
                "explore" => {
                    let mut matches = matches.clone();
                    let selected_note = matches
                        .remove_many::<String>("select")
                        .map(|elems| elems.collect::<Vec<_>>());
                    commands::explore::exec(
                        db,
                        selected_note,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                        surf_bindings,
                        checkmark_bindings,
                        stack_bindings,
                        explore_bindings,
                    )
                    .await
                }
                "link" => {
                    commands::link::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "surf" => {
                    commands::surf::exec(
                        db,
                        config.surf_parsing,
                        config.external_commands,
                        md_static,
                        config.color.elements,
                        surf_bindings,
                        explore_bindings,
                    )
                    .await
                }
                "unlink" => {
                    commands::unlink::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "remove" => {
                    commands::remove::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "rename" => {
                    commands::rename::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "print" => {
                    let name = matches.get_one::<String>("name").cloned();
                    commands::print::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        name,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "select" => {
                    commands::select::exec(
                        db,
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                    )
                    .await
                }
                "checkmark" => {
                    commands::checkmark::exec(
                        db,
                        config.surf_parsing,
                        config.external_commands,
                        md_static,
                        config.color.elements,
                        checkmark_bindings,
                        explore_bindings,
                    )
                    .await
                }
                "stack" => {
                    commands::stack::exec(
                        db,
                        vec![],
                        config.external_commands,
                        config.surf_parsing,
                        md_static,
                        config.color.elements,
                        stack_bindings,
                    )
                    .await
                }
                _ => unreachable!("clap should ensure we don't get here"),
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
    result
}
