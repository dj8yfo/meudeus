use std::borrow::Cow;

use colored::Colorize;
use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::{
    external_commands::{fetch_content, list_dir},
    highlight::highlight_code_block,
};

impl SkimItem for super::Link {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!("{}", self))
    }
    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        let input = self.skim_display();
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

            super::Destination::CodeBlock {
                code_block,
                syntax_label,
                ..
            } => ItemPreview::AnsiText(highlight_code_block(code_block, syntax_label)),
        }
    }
}
