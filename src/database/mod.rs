use async_trait::async_trait;
use sqlx::Result;

use crate::{config::color::ColorScheme, highlight::MarkdownStatic, note::Note};

mod sqlite;
pub use sqlite::{Sqlite, SqliteAsyncHandle};

#[async_trait]
pub trait Database: Send + Sync {
    async fn save(&mut self, note: &Note) -> Result<()>;
    async fn list(&self, md_static: MarkdownStatic, color_scheme: ColorScheme)
        -> Result<Vec<Note>>;
    async fn get(
        &self,
        name: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Note>;
    async fn remove_note(&mut self, note: &Note) -> Result<()>;
    async fn rename_note(&mut self, note: &Note, new_name: &str) -> Result<()>;
    async fn insert_link(&mut self, from: &str, to: &str, straight: bool) -> Result<()>;
    async fn remove_link(&mut self, from: &str, to: &str, straight: bool) -> Result<()>;
    async fn find_links_from(
        &self,
        from: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> Result<Vec<Note>>;
    async fn find_links_to(
        &self,
        to: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> Result<Vec<Note>>;
    async fn push_note_to_stack(&mut self, stack: &str, note: &str) -> Result<()>;
    async fn select_from_stack(
        &mut self,
        stack: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Vec<Note>>;
    async fn pop_note_from_stack(&mut self, stack: &str, note: &str) -> Result<()>;
    async fn move_to_topmost(&mut self, stack: &str, note: &str) -> Result<()>;
}
