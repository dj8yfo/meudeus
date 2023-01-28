use std::io::Write;
use std::{fs::File, io, path::PathBuf};

use crate::{database::SqliteAsyncHandle, dir::Directory};
mod random;
mod skim_item;

#[derive(Clone, Debug)]
pub struct AsyncQeuryResources {
    pub db: SqliteAsyncHandle,
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
    pub(crate) fn init(name: String, dir: Directory, is_tag: bool) -> Self {
        let time_str = chrono::Utc::now().naive_utc().timestamp().to_string();
        let suffix = random::rand_suffix();
        let fname = format!("{}_{}.md", time_str, suffix);

        let file_path = (!is_tag).then_some(dir.path.join(fname));
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

    pub fn resources(&self) -> Option<&AsyncQeuryResources> {
        match self {
            Self::MdFile { resources, .. } => resources.as_ref(),
            Self::Tag { resources, .. } => resources.as_ref(),
        }
    }
}
