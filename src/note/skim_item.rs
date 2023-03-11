use std::borrow::Cow;

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use super::PreviewType;

mod preview;

impl SkimItem for super::Note {
    fn text(&self) -> Cow<str> {
        Cow::Owned(self.name())
    }
    fn display<'a>(&'a self, _context: DisplayContext<'a>) -> AnsiString<'a> {
        let input = format!("{}", self);
        let ansistring = AnsiString::parse(&input);
        ansistring
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        match self.resources() {
            Some(resources) => match resources.preview_type {
                PreviewType::Details => self.details(),
                PreviewType::Structure => self.structure(),
            },
            None => ItemPreview::Text("<empty>".to_string()),
        }
    }
}
