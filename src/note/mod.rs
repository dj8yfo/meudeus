use colored::Colorize;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::{fs::File, io, path::PathBuf};

use crate::config::Open as OpenCfg;
use crate::config::{ExternalCommands, SurfParsing};
use crate::database::SqliteAsyncHandle;
use crate::highlight::highlight_code_block;
use crate::Open;
mod links_term_tree;
mod random;
mod reachable;
mod skim_item;
mod task_items_term_tree;
pub use self::task_items_term_tree::NoteTaskItemTerm;
use crate::database::Database;
use duct::cmd;
use sqlx::Result as SqlxResult;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum PreviewType {
    Details,
    LinkStructure,
    TaskStructure,
}

impl PreviewType {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Details => Self::LinkStructure,
            Self::LinkStructure => Self::TaskStructure,
            Self::TaskStructure => Self::Details,
        }
    }
}

impl Default for PreviewType {
    fn default() -> Self {
        Self::LinkStructure
    }
}

#[derive(Clone, Debug)]
pub struct DynResources {
    pub external_commands: ExternalCommands,
    pub surf_parsing: SurfParsing,
    pub preview_type: PreviewType,
    pub preview_result: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Note {
    MdFile {
        name: String,
        file_path: PathBuf,
        resources: Option<DynResources>,
        name_markdown: Option<String>,
    },
    Tag {
        name: String,
        resources: Option<DynResources>,
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
        match self {
            Self::MdFile { name_markdown, .. } => {
                let md = name_markdown.as_ref().cloned().unwrap();
                write!(f, "{}", md)
            }

            Self::Tag { name, .. } => {
                if name == "METATAG" || name == "root" {
                    write!(f, "{}", name.red().to_string())
                } else {
                    write!(f, "{}", name.cyan().to_string())
                }
            }
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
            Some(file_path) => {
                Self::MdFile {
                    name,
                    file_path,
                    resources: None,
                    name_markdown: None,
                }
            }
            None => Self::Tag {
                name,
                resources: None,
            },
        }
    }

    pub(crate) fn set_name(&mut self) {
        match self {
            Self::MdFile {
                name, 
                ref mut name_markdown,
                ..
            } => {

                let markdown = format!(
                    "{} {}",
                    highlight_code_block(&name, "markdown"),
                    " ".black().to_string()
                );
                *name_markdown = Some(markdown);
            }
            Self::Tag {
                ..
            } => {
                // nothing
            }
        }
    }

    pub(crate) fn init(name: String, is_tag: bool) -> Self {
        let time_str = chrono::Utc::now().naive_utc().timestamp().to_string();
        let suffix = random::rand_suffix();
        let fname = format!("{}_{}.md", time_str, suffix);

        let file_path = (!is_tag).then_some(PathBuf::from("./").join(fname));
        let mut note = Self::new(name, file_path);
        note.set_name();
        note
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
    pub fn set_resources(&mut self, to_set: DynResources) {
        match self {
            Self::MdFile { resources, .. } => {
                *resources = Some(to_set);
            }
            Self::Tag { resources, .. } => {
                *resources = Some(to_set);
            }
        }
    }

    fn resources_mut(&mut self) -> Option<&mut DynResources> {
        match self {
            Self::MdFile {
                ref mut resources, ..
            } => resources.as_mut(),
            Self::Tag {
                ref mut resources, ..
            } => resources.as_mut(),
        }
    }
    fn resources(&self) -> Option<&DynResources> {
        match self {
            Self::MdFile { ref resources, .. } => resources.as_ref(),
            Self::Tag { ref resources, .. } => resources.as_ref(),
        }
    }

    pub async fn fetch_forward_links(&self, db: &SqliteAsyncHandle) -> SqlxResult<Vec<Note>> {
        db.lock().await.find_links_from(&self.name()).await
    }

    pub async fn fetch_backlinks(&self, db: &SqliteAsyncHandle) -> SqlxResult<Vec<Note>> {
        db.lock().await.find_links_to(&self.name()).await
    }
}
