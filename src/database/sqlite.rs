use std::{fs, path::Path, str::FromStr, sync::Arc};

use async_std::sync::Mutex;
use async_trait::async_trait;
use sql_builder::{SqlBuilder, SqlName};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow},
    Result, Row, SqlitePool,
};

use crate::note::Note;

use super::Database;

#[derive(Debug)]
pub struct Sqlite {
    pool: SqlitePool,
}

pub type SqliteAsyncHandle = Arc<Mutex<Sqlite>>;

impl Sqlite {
    pub async fn new(create_if_missing: bool, path: impl AsRef<Path>) -> Result<Arc<Mutex<Self>>> {
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

        Ok(Arc::new(Mutex::new(Self { pool })))
    }

    async fn setup_db(pool: &SqlitePool) -> Result<()> {
        log::debug!("running sqlite database setup");

        sqlx::migrate!("./migrations").run(pool).await?;

        Ok(())
    }

    async fn save_note(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, note: &Note) -> Result<()> {
        let path: Option<&str> = note
            .file_path()
            .map(|path| path.as_path().to_str().unwrap());
        sqlx::query(
            "insert into notes(name, filename)
                values(?1, ?2)",
        )
        .bind(note.name().as_str())
        .bind(path)
        .execute(tx)
        .await?;

        Ok(())
    }
    fn query_note(row: SqliteRow) -> Note {
        let file_path: Option<String> = row.get("filename");
        Note::new(row.get("name"), file_path.map(|c| c.into()))
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

    async fn list(&self) -> Result<Vec<Note>> {
        log::debug!("listing notes");

        let mut query = SqlBuilder::select_from(SqlName::new("notes").alias("n").baquoted());
        query.field("*").order_desc("name");

        let query = query.sql().expect("bug in list query. please report");

        let res = sqlx::query(&query)
            .map(Self::query_note)
            .fetch_all(&self.pool)
            .await?;

        Ok(res)
    }
}
