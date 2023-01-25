use colored::Colorize;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct Directory {
    pub(crate) path: PathBuf,
}

impl Directory {
    pub(crate) fn new(path: Option<&PathBuf>) -> Self {
        let path = if let Some(path) = path {
            path.clone()
        } else {
            let path = PathBuf::from("./__default_home_path__");
            println!("{}", format!("using default path {:?}", path).cyan());
            path
        };
        Self { path }
    }

    pub fn check(&self) -> anyhow::Result<()> {
        if !self.path.as_path().is_dir() {
            return Err(anyhow::anyhow!("not a dir {}", self.path.display()));
        } // Check if able to write inside directory
        let md = fs::metadata(self.path.as_path())?;
        let permissions = md.permissions();
        let readonly = permissions.readonly();
        if readonly {
            return Err(anyhow::anyhow!(
                "no write permissions for self {}",
                self.path.display()
            ));
        }
        Ok(())
    }
}
