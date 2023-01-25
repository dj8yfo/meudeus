use async_trait::async_trait;
use sqlx::Result;

use crate::note::Note;

mod sqlite;
pub use sqlite::{Sqlite, SqliteAsyncHandle};

#[async_trait]
pub trait Database: Send + Sync {
    async fn save(&mut self, note: &Note) -> Result<()>;
    async fn list(&self) -> Result<Vec<Note>>;
}
