use std::{path::PathBuf, fs::File, io};


use crate::dir::Directory;
mod random;

#[derive(Clone, Debug)]
pub struct Note {
    pub name: String,
    pub file_path: Option<PathBuf>,
}
impl Note {
    pub(crate) fn init(name: String, dir: Directory, is_tag: bool) -> Self {
        let time_str = chrono::Utc::now().naive_utc().timestamp().to_string();
        let suffix = random::rand_suffix();
        let fname = format!("{}_{}.md", time_str, suffix);
        
        let file_path = (!is_tag).then_some(dir.path.join(fname));
        Self { name, file_path }
    }

    pub(crate) fn persist(&self) -> Result<(), io::Error> {

        if let Some(file_path) = &self.file_path {
            File::create(file_path.as_path())?;
        }
        Ok(())
    }
}
