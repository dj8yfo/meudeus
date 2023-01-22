use std::{fs, path::Path, str::FromStr};

use async_trait::async_trait;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Result, SqlitePool,
};

use crate::note::Note;

use super::Database;

pub struct Sqlite {
    pool: SqlitePool,
}
impl Sqlite {
    pub async fn new(create_if_missing: bool, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        log::debug!("opening sqlite database at {:?}", path);

        let create = !path.exists();
        if create {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
        }

        let opts = SqliteConnectOptions::from_str(path.as_os_str().to_str().unwrap())?
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(create_if_missing);

        let pool = SqlitePoolOptions::new().connect_with(opts).await?;

        Self::setup_db(&pool).await?;

        Ok(Self { pool })
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        log::debug!("running sqlite database setup");

        sqlx::migrate!("./migrations").run(pool).await?;

        Ok(())
    }

    async fn save_note(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, note: &Note) -> Result<()> {
        let path: Option<&str> = note
            .file_path.as_ref()
            .map(|path| path.as_path().to_str().unwrap());
        sqlx::query(
            "insert into notes(name, filename)
                values(?1, ?2)",
        )
        .bind(note.name.as_str())
        .bind(path)
        .execute(tx)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl Database for Sqlite {
    async fn save(&mut self, note: &Note) -> Result<()> {
        log::debug!("saving note to sqlite {:?}", note);

        let mut tx = self.pool.begin().await?;
        Self::save_note(&mut tx, note).await?;
        tx.commit().await?;

        Ok(())
    }
}
