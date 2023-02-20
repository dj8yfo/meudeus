use std::borrow::Cow;

use colored::Colorize;
use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::external_commands::{fetch_content, list_dir};

impl SkimItem for super::Link {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!("{}", self))
    }
    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        let parent_name = self.parent_name.yellow().to_string();
        let description = match self.link {
            super::Destination::URL(..) => self.description.green().to_string(),
            super::Destination::File { .. } => self.description.cyan().to_string(),
            super::Destination::Dir { .. } => self.description.magenta().to_string(),
            super::Destination::Broken(..) => self.description.red().to_string(),
        };

        let input = format!("{} -> [{}]", parent_name, description);

        let ansistring = AnsiString::parse(&input);
        ansistring
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        match &self.link {
            super::Destination::URL(url) => ItemPreview::AnsiText(url.cyan().to_string()),
            super::Destination::File { file, preview } => {
                ItemPreview::AnsiText(fetch_content(preview.clone(), Some(file)).unwrap())
            }
            super::Destination::Dir { dir, preview } => {
                ItemPreview::AnsiText(list_dir(preview.clone(), dir))
            }
            super::Destination::Broken(broken) => ItemPreview::AnsiText(format!(
                "{}: {}",
                "Broken path",
                broken
                    .to_str()
                    .unwrap_or("not valid unicode")
                    .red()
                    .to_string(),
            )),
        }
    }
}
