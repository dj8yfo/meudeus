use colored::Colorize;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::{fs::File, io, path::PathBuf};

use crate::config::Open as OpenCfg;
use crate::config::{ExternalCommands, SurfParsing};
use crate::database::SqliteAsyncHandle;
use crate::Open;
mod parse_link;
mod random;
mod reachable;
mod skim_item;
use crate::database::Database;
use duct::cmd;
use sqlx::Result as SqlxResult;

#[derive(Clone, Debug, Copy)]
pub enum PreviewType {
    Details,
    Structure,
}

impl PreviewType {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Details => Self::Structure,
            Self::Structure => Self::Details,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AsyncQeuryResources {
    pub db: SqliteAsyncHandle,
    pub external_commands: ExternalCommands,
    pub surf_parsing: SurfParsing,
    pub preview_type: PreviewType,
}

#[derive(Clone, Debug)]
pub enum Note {
    MdFile {
        name: String,
        file_path: PathBuf,
        resources: Option<AsyncQeuryResources>,
    },
    Tag {
        name: String,
        resources: Option<AsyncQeuryResources>,
    },
}

impl Hash for Note {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file_path().is_none() {
            if self.name() == "METATAG" || self.name() == "root" {
                write!(f, "{}", self.name().red().to_string())
            } else {
                write!(f, "{}", self.name().cyan().to_string())
            }
        } else {
            write!(f, "{}", self.name().yellow().to_string())
        }
    }
}

impl Open for Note {
    fn open(&self, mut cfg: OpenCfg) -> io::Result<Option<std::process::ExitStatus>> {
        if let Some(file_path) = self.file_path() {
            cfg.file_cmd
                .replace_matching_element("$FILE", file_path.to_str().unwrap_or("bad utf path"));
            Ok(Some(
                cmd(cfg.file_cmd.command, cfg.file_cmd.args).run()?.status,
            ))
        } else {
            Ok(None)
        }
    }
}

impl Eq for Note {}

impl Note {
    pub(crate) fn new(name: String, file_path: Option<PathBuf>) -> Self {
        match file_path {
            Some(file_path) => Self::MdFile {
                name,
                file_path,
                resources: None,
            },
            None => Self::Tag {
                name,
                resources: None,
            },
        }
    }
    pub(crate) fn init(name: String, is_tag: bool) -> Self {
        let time_str = chrono::Utc::now().naive_utc().timestamp().to_string();
        let suffix = random::rand_suffix();
        let fname = format!("{}_{}.md", time_str, suffix);

        let file_path = (!is_tag).then_some(PathBuf::from("./").join(fname));
        Self::new(name, file_path)
    }

    pub(crate) fn persist(&self) -> Result<(), io::Error> {
        if let Self::MdFile { file_path, .. } = &self {
            let mut output = File::create(file_path.as_path())?;
            writeln!(output, "# 💖 {}", self.name())?;
        }
        Ok(())
    }
    pub fn name(&self) -> String {
        match &self {
            Self::MdFile { name, .. } => name.clone(),
            Self::Tag { name, .. } => name.clone(),
        }
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            Self::MdFile { file_path, .. } => Some(file_path),
            Self::Tag { .. } => None,
        }
    }
    pub fn set_resources(&mut self, to_set: AsyncQeuryResources) {
        match self {
            Self::MdFile { resources, .. } => {
                *resources = Some(to_set);
            }
            Self::Tag { resources, .. } => {
                *resources = Some(to_set);
            }
        }
    }

    fn resources(&self) -> Option<&AsyncQeuryResources> {
        match self {
            Self::MdFile { resources, .. } => resources.as_ref(),
            Self::Tag { resources, .. } => resources.as_ref(),
        }
    }

    pub async fn fetch_forward_links(&self) -> Option<SqlxResult<Vec<Note>>> {
        if let Some(resources) = self.resources() {
            Some(
                resources
                    .db
                    .lock()
                    .await
                    .find_links_from(&self.name())
                    .await,
            )
        } else {
            None
        }
    }

    pub async fn fetch_backlinks(&self) -> Option<SqlxResult<Vec<Note>>> {
        if let Some(resources) = self.resources() {
            Some(resources.db.lock().await.find_links_to(&self.name()).await)
        } else {
            None
        }
    }
}
