use std::borrow::Cow;

use colored::Colorize;
use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::{
    external_commands::{fetch_content, list_dir},
    highlight::highlight_code_block,
};

impl super::Link {
    pub fn prepare_display(&mut self) {
        let input = self.skim_display();
        self.display_item = Some(AnsiString::parse(&input));
    }

    pub fn compute_preview(&self) -> String {
        match &self.link {
            super::Destination::URL(url) => url.cyan().to_string(),
            super::Destination::File { file, preview } => {
                fetch_content(preview.clone(), Some(file)).unwrap()
            }
            super::Destination::Dir { dir, preview } => {
                list_dir(preview.clone(), dir)
            }
            super::Destination::Broken(broken) => format!(
                "{}: {}",
                "Broken path",
                broken
                    .to_str()
                    .unwrap_or("not valid unicode")
                    .red()
                    .to_string(),
            ),

            super::Destination::CodeBlock {
                code_block,
                syntax_label,
                ..
            } => highlight_code_block(code_block, syntax_label),
        }
    }

    pub fn prepare_preview(&mut self) {
        let result = self.compute_preview();
        self.preview_item = Some(result);
    }
}

impl SkimItem for super::Link {
    fn text(&self) -> Cow<str> {
        Cow::Owned(format!("{}", self))
    }
    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        if let Some(ref string) = self.display_item {
            string.clone()
        } else {
            AnsiString::parse("<not precomputed!!!>")
        }
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        if let Some(ref string) = self.preview_item {
            return ItemPreview::AnsiText(string.clone());
        } else {
            ItemPreview::AnsiText("<not precomputed!!!>".to_string())
        }
        
    }
}
