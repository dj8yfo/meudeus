use colored::Colorize;
use std::borrow::Cow;
use std::process::Command;

use skim::{ItemPreview, PreviewContext, SkimItem};

impl SkimItem for super::Note {
    fn text(&self) -> Cow<str> {
        Cow::Owned(self.name())
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        let mut string = String::new();
        string.push_str(&"Note metadata...\n \n".cyan().to_string());
        let body = fetch_content(self);
        string.push_str(&body);
        ItemPreview::AnsiText(string)
    }
}

fn fetch_content(note: &super::Note) -> String {
    if let Some(file_path) = note.file_path() {
        match Command::new("bat")
            .arg("--color=always")
            .arg(file_path.as_os_str())
            .output()
        {
            Ok(output) => match String::from_utf8(output.stdout) {
                Ok(string) => string,
                Err(err) => format!("{:?}", err).red().to_string(),
            },
            Err(err) => format!("{:?}", err).red().to_string(),
        }
    } else {
        format!("it's a tag: {}", note.name()).green().to_string()
    }
}
