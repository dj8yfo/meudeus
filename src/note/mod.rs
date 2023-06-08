use colored::Colorize;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::{fs::File, io, path::PathBuf};
use syntect::easy::HighlightLines;

use crate::config::color::ColorScheme;
use crate::config::Open as OpenCfg;
use crate::config::{ExternalCommands, SurfParsing};
use crate::database::SqliteAsyncHandle;
use crate::highlight::{highlight, MarkdownStatic};
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
        color_scheme: ColorScheme,
    },
    Tag {
        name: String,
        resources: Option<DynResources>,
        color_scheme: ColorScheme,
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

            Self::Tag {
                name, color_scheme, ..
            } => {
                if name == "METATAG" || name == "root" {
                    let special_tag = color_scheme.notes.special_tag;
                    write!(
                        f,
                        "{}",
                        name.truecolor(special_tag.0.r, special_tag.0.g, special_tag.0.b)
                    )
                } else {
                    let tag = color_scheme.notes.tag;
                    write!(f, "{}", name.truecolor(tag.0.r, tag.0.g, tag.0.b))
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

    fn open_xdg(&self) -> Result<(), opener::OpenError> {
        if let Some(file_path) = self.file_path() {
            opener::open(file_path)
        } else {
            Ok(())
        }
    }
}

impl Eq for Note {}

impl Note {
    pub(crate) fn new(name: String, file_path: Option<PathBuf>, color_scheme: ColorScheme) -> Self {
        match file_path {
            Some(file_path) => Self::MdFile {
                name,
                file_path,
                resources: None,
                name_markdown: None,
                color_scheme,
            },
            None => Self::Tag {
                name,
                resources: None,
                color_scheme,
            },
        }
    }
    pub(crate) fn rename(
        &mut self,
        new_name: &str,
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,
    ) {
        match self {
            Self::MdFile { ref mut name, .. } => {
                *name = new_name.to_string();
            }
            Self::Tag { ref mut name, .. } => {
                *name = new_name.to_string();
            }
        }
        self.set_markdown(highlighter, md_static);
    }

    pub(crate) fn set_markdown(
        &mut self,
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,
    ) {
        match self {
            Self::MdFile {
                name,
                ref mut name_markdown,
                ..
            } => {
                let markdown = format!(
                    "{} {}",
                    highlight(name, highlighter, md_static),
                    " ".truecolor(0, 0, 0)
                );
                *name_markdown = Some(markdown);
            }
            Self::Tag { .. } => {
                // nothing
            }
        }
    }

    pub(crate) fn init(
        name: String,
        is_tag: bool,
        highlighter: &mut HighlightLines,
        md_static: MarkdownStatic,

        color_scheme: ColorScheme,
    ) -> Self {
        let time_str = chrono::Utc::now().naive_utc().timestamp().to_string();
        let suffix = random::rand_suffix();
        let fname = format!("{}_{}.md", time_str, suffix);

        let file_path = (!is_tag).then_some(PathBuf::from("./").join(fname));
        let mut note = Self::new(name, file_path, color_scheme);
        note.set_markdown(highlighter, md_static);
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

    pub async fn fetch_forward_links(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> SqlxResult<Vec<Note>> {
        db.lock()
            .await
            .find_links_from(&self.name(), md_static, color_scheme, straight)
            .await
    }

    pub async fn fetch_backlinks(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> SqlxResult<Vec<Note>> {
        db.lock()
            .await
            .find_links_to(&self.name(), md_static, color_scheme, straight)
            .await
    }
}
