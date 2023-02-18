use std::{fmt::Display, io, path::PathBuf, process::Command};

use regex::Regex;
use duct::cmd;

use crate::{print::print_two_tokens, Open};
mod skim_item;

#[derive(Clone, Debug)]
pub enum Destination {
    URL(String),
    File(PathBuf),
    Dir(PathBuf),
    Broken(PathBuf),
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::URL(url) => write!(f, "{}", url),
            Self::File(file) => write!(f, "{}", file.display()),
            Self::Dir(dir) => write!(f, "{}", dir.display()),
            Self::Broken(broken) => write!(f, "{}", broken.display()),
        }
    }
}

impl Open for Link {
    fn open(&self) -> io::Result<std::process::ExitStatus> {
        match &self.link {
            Destination::URL(url) => Command::new("firefox").arg(url).status(),

            Destination::File(file) => Command::new("helix-22.12-x86_64.AppImage")
                .arg(file)
                .status(),
            Destination::Dir(dir) => {

                let args = vec![
                    "action",
                    "new-pane",
                    "--cwd",
                    dir.to_str().unwrap_or("bad utf path"),
                    "--",
                    "broot",
                ];
                Ok(cmd("zellij", args).run()?.status)
            }
            Destination::Broken(broken) => {
                println!(
                    "{}",
                    print_two_tokens(
                        "cannot open broken: ",
                        broken.to_str().unwrap_or("bad utf8")
                    )
                );
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not possible for broken path",
                ))
            }
        }
    }
}
#[derive(Clone, Debug)]
pub struct Link {
    pub parent_name: String,
    pub description: String,
    pub link: Destination,
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:: {} :{}",
            self.parent_name, self.description, self.link
        )
    }
}
lazy_static! {
    static ref URL: Regex = Regex::new(r#"^https?://\S+"#).unwrap();
}

impl Link {
    pub fn new(
        description: String,
        link: String,
        parent_note: PathBuf,
        parent_name: String,
    ) -> Self {
        if URL.is_match(&link) {
            Self {
                parent_name,
                description,
                link: Destination::URL(link),
            }
        } else {
            let mut link = PathBuf::from(&link);
            if link.is_relative() {
                link = parent_note.as_path().parent().unwrap().join(link);
            }
            if link.is_file() {
                Self {
                    parent_name,
                    description,
                    link: Destination::File(link),
                }
            } else if link.is_dir() {
                Self {
                    parent_name,
                    description,
                    link: Destination::Dir(link),
                }
            } else {
                Self {
                    parent_name,
                    description,
                    link: Destination::Broken(link),
                }
            }
        }
    }
}
