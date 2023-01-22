use sqlx::Result;
use async_trait::async_trait;

use crate::note::Note;

mod sqlite;
pub use sqlite::Sqlite;

#[async_trait]
pub trait Database: Send + Sync {
    async fn save(&mut self, note: &Note) -> Result<()>;
}
