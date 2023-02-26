use async_trait::async_trait;
use sqlx::Result;

use crate::note::Note;

mod sqlite;
pub use sqlite::{Sqlite, SqliteAsyncHandle};

#[async_trait]
pub trait Database: Send + Sync {
    async fn save(&mut self, note: &Note) -> Result<()>;
    async fn list(&self) -> Result<Vec<Note>>;
    async fn insert_link(&mut self, from: &str, to: &str) -> Result<()>;
    async fn remove_link(&mut self, from: &str, to: &str) -> Result<()>;
    async fn find_links_from(&self, from: &str) -> Result<Vec<Note>>;
    async fn find_links_to(&self, to: &str) -> Result<Vec<Note>>;
}
