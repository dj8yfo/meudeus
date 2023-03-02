use std::borrow::Cow;

use colored::Colorize;
use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::external_commands::{fetch_content, list_dir};

use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

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
            super::Destination::CodeBlock { .. } => self.description.blue().to_string(),
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

            super::Destination::CodeBlock {
                code_block,
                syntax_label,
                ..
            } => ItemPreview::AnsiText(highlight_code_block(code_block.clone(), syntax_label)),
        }
    }
}

pub(super) fn highlight_code_block(input: String, syntax_desc: &str) -> String {
    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_token(syntax_desc);
    if let Some(syntax) = syntax {
        let mut result = String::new();

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-eighties.dark"]);
        for line in LinesWithEndings::from(&input) {
            // LinesWithEndings enables use of newlines mode
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            result.push_str(&escaped);
        }
        result
    } else {
        input
    }
}
