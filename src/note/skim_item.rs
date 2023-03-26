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
            Some(resources) => {
                if let Some(val) = resources.cached_preview_result.lock().expect("poison").get(&resources.preview_type) {
                    return ItemPreview::AnsiText(val.clone());
                }

                let (preview, result) = match resources.preview_type {
                    preview @ PreviewType::Details => (preview, self.details()),
                    preview @ PreviewType::LinkStructure => (preview, self.link_structure()),
                    preview @ PreviewType::TaskStructure => (preview, self.task_structure()),
                };
                resources.cached_preview_result.lock().expect("poison").insert(preview, result.clone());
                ItemPreview::AnsiText(result)
            }
            None => ItemPreview::Text("<empty>".to_string()),
        }
    }
}
