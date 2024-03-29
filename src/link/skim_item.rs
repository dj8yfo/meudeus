use colored::Colorize;
use std::borrow::Cow;

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::{
    config::{color::ColorScheme, Preview},
    external_commands::{fetch_content, fetch_content_range, list_dir},
    highlight::{highlight_code_block, MarkdownStatic},
};

impl super::Link {
    pub fn prepare_display(&mut self) {
        let input = self.skim_display();
        self.display_item = Some(AnsiString::parse(&input));
    }

    pub fn compute_preview(
        &self,
        preview_cmds: &Preview,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> String {
        match &self.link {
            super::Destination::Url(url) => {
                let c = color_scheme.links.url;
                url.truecolor(c.0.r, c.0.g, c.0.b).to_string()
            }
            super::Destination::File { file } => {
                fetch_content(preview_cmds.file_cmd.clone(), Some(file)).unwrap()
            }
            super::Destination::FileLine { file, line_number } => {
                fetch_content_range(preview_cmds.file_line_cmd.clone(), Some(file), *line_number)
                    .unwrap()
            }
            super::Destination::Dir { dir } => list_dir(preview_cmds.dir_cmd.clone(), dir),
            super::Destination::Broken(broken, line) => {
                let line = if let Some(line) = line {
                    format!("<line:{}>", line)
                } else {
                    String::new()
                };
                let c = color_scheme.links.broken;
                format!(
                    "{}: {} {}",
                    "Broken path",
                    broken
                        .to_str()
                        .unwrap_or("not valid unicode")
                        .truecolor(c.0.r, c.0.g, c.0.b),
                    line,
                )
            }

            super::Destination::CodeBlock {
                code_block,
                syntax_label,
                ..
            } => highlight_code_block(code_block, syntax_label, md_static),
        }
    }

    pub fn prepare_preview(
        &mut self,
        preview_cmds: &Preview,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) {
        let result = self.compute_preview(preview_cmds, md_static, color_scheme);
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
            ItemPreview::AnsiText(string.clone())
        } else {
            ItemPreview::AnsiText("<not precomputed!!!>".to_string())
        }
    }
}
