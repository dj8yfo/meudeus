use std::borrow::Cow;

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::{config::color::ColorScheme, database::SqliteAsyncHandle, highlight::MarkdownStatic};

use super::PreviewType;

mod preview;

impl super::Note {
    async fn compute_preview(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
        nested_threshold: usize,
    ) -> Option<String> {
        match self.resources() {
            Some(resources) => {
                let result = match resources.preview_type {
                    PreviewType::Details => {
                        self.details(db, md_static, color_scheme, straight).await
                    }
                    PreviewType::LinkStructure => {
                        self.link_structure(db, md_static, color_scheme, straight, nested_threshold)
                            .await
                    }
                    PreviewType::TaskStructure => {
                        self.task_structure(db, md_static, color_scheme, straight, nested_threshold)
                            .await
                    }
                };
                Some(result)
            }
            None => None,
        }
    }
    pub async fn prepare_preview(
        &mut self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
        nested_threshold: usize,
    ) {
        let result = self
            .compute_preview(db, md_static, color_scheme, straight, nested_threshold)
            .await;
        match self.resources_mut() {
            Some(resources) => {
                resources.preview_result = result;
            }
            None => {}
        }
    }
}

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
                if let Some(ref result) = resources.preview_result {
                    ItemPreview::AnsiText(result.clone())
                } else {
                    ItemPreview::Text("<empty preview_result>".to_string())
                }
            }
            None => ItemPreview::Text("<empty resources>".to_string()),
        }
    }
}
