use std::{fs, path::Path, str::FromStr, sync::Arc};

use async_std::sync::Mutex;
use async_trait::async_trait;
use sql_builder::{quote, SqlBuilder, SqlName};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow},
    Error, Result, Row, SqlitePool,
};
use syntect::easy::HighlightLines;

use crate::{config::color::ColorScheme, highlight::MarkdownStatic, note::Note};

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

    async fn insert_link(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        from: &str,
        to: &str,
    ) -> Result<()> {
        sqlx::query(
            "insert into linkx(_from, _to)
                values(?1, ?2)",
        )
        .bind(from)
        .bind(to)
        .execute(tx)
        .await?;

        Ok(())
    }

    async fn remove_link(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        from: &str,
        to: &str,
    ) -> Result<()> {
        sqlx::query(
            "delete from linkx
            where _from = ?1 and _to = ?2",
        )
        .bind(from)
        .bind(to)
        .execute(tx)
        .await?;

        Ok(())
    }

    async fn remove_note(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, note: &Note) -> Result<()> {
        sqlx::query(
            "delete from notes
            where name = ?1",
        )
        .bind(note.name().as_str())
        .execute(tx)
        .await?;

        Ok(())
    }

    async fn rename_note(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        note: &Note,
        new_name: &str,
    ) -> Result<()> {
        sqlx::query("update notes set name = ?2 where name = ?1")
            .bind(note.name().as_str())
            .bind(new_name)
            .execute(tx)
            .await?;

        Ok(())
    }

    fn query_note(row: SqliteRow, color_scheme: ColorScheme) -> Note {
        let file_path: Option<String> = row.get("filename");
        Note::new(row.get("name"), file_path.map(|c| c.into()), color_scheme)
    }

    async fn find_links_from_inner(
        &self,
        from: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Vec<Note>> {
        log::debug!("listing notes, linked by current");

        let sql = SqlBuilder::select_from(name!("linkx"; "l"))
            .field(name!("l", "_to"; "name"))
            .field(name!("n", "filename"; "filename"))
            .left()
            .join(name!("notes"; "n"))
            .on("l._to = n.name")
            .and_where_eq("l._from", quote(from))
            .order_desc("l._to")
            .sql()
            .map_err(|err| Error::Protocol(format!("{:?}", err)))?;
        log::debug!("sql: {}", sql);

        let res = sqlx::query(&sql)
            .map(|row| Self::query_note(row, color_scheme))
            .fetch_all(&self.pool)
            .await?;
        let res = parse_names(res, md_static).await;

        Ok(res)
    }

    async fn find_links_to_inner(
        &self,
        to: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Vec<Note>> {
        log::debug!("listing notes, linked by current");

        let sql = SqlBuilder::select_from(name!("linkx"; "l"))
            .field(name!("l", "_from"; "name"))
            .field(name!("n", "filename"; "filename"))
            .left()
            .join(name!("notes"; "n"))
            .on("l._from = n.name")
            .and_where_eq("l._to", quote(to))
            .order_desc("l._from")
            .sql()
            .map_err(|err| Error::Protocol(format!("{:?}", err)))?;
        log::debug!("sql: {}", sql);

        let res = sqlx::query(&sql)
            .map(|row| Self::query_note(row, color_scheme))
            .fetch_all(&self.pool)
            .await?;
        let res = parse_names(res, md_static).await;

        Ok(res)
    }

    async fn insert_link_inner(&mut self, from: &str, to: &str) -> Result<()> {
        log::debug!("saving link {} -> {} ", from, to);

        let mut tx = self.pool.begin().await?;
        Self::insert_link(&mut tx, from, to).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn remove_link_inner(&mut self, from: &str, to: &str) -> Result<()> {
        log::debug!("removing link {} -> {} ", from, to);

        let mut tx = self.pool.begin().await?;
        Self::remove_link(&mut tx, from, to).await?;
        tx.commit().await?;

        Ok(())
    }
}

async fn parse_names(notes: Vec<Note>, md_static: MarkdownStatic) -> Vec<Note> {
    let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
    notes
        .into_iter()
        .map(|mut note| {
            note.set_markdown(&mut highlighter, md_static);
            note
        })
        .collect::<Vec<_>>()
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

    async fn list(
        &self,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Vec<Note>> {
        log::debug!("listing notes");

        let mut query = SqlBuilder::select_from(SqlName::new("notes").alias("n").baquoted());
        query.field("*").order_desc("name");

        let query = query.sql().expect("bug in list query. please report");

        let res = sqlx::query(&query)
            .map(|row| Self::query_note(row, color_scheme))
            .fetch_all(&self.pool)
            .await?;
        let res = parse_names(res, md_static).await;

        Ok(res)
    }

    async fn get(
        &self,
        name: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> Result<Note> {
        log::debug!("listing notes");

        let mut query = SqlBuilder::select_from(SqlName::new("notes").alias("n").baquoted());
        query.field("*").and_where_eq("name", &quote(name));

        let query = query.sql().expect("bug in list query. please report");

        let mut res = sqlx::query(&query)
            .map(|row| Self::query_note(row, color_scheme))
            .fetch_one(&self.pool)
            .await?;
        let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
        res.set_markdown(&mut highlighter, md_static);

        Ok(res)
    }

    async fn find_links_from(
        &self,
        from: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> Result<Vec<Note>> {
        if straight {
            self.find_links_from_inner(from, md_static, color_scheme)
                .await
        } else {
            self.find_links_to_inner(from, md_static, color_scheme)
                .await
        }
    }

    async fn find_links_to(
        &self,
        to: &str,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> Result<Vec<Note>> {
        if straight {
            self.find_links_to_inner(to, md_static, color_scheme).await
        } else {
            self.find_links_from_inner(to, md_static, color_scheme)
                .await
        }
    }

    async fn insert_link(&mut self, mut from: &str, mut to: &str, straight: bool) -> Result<()> {
        if !straight {
            let tmp = from;
            from = to;
            to = tmp;
        }
        self.insert_link_inner(from, to).await
    }

    async fn remove_link(&mut self, mut from: &str, mut to: &str, straight: bool) -> Result<()> {
        if !straight {
            let tmp = from;
            from = to;
            to = tmp;
        }
        self.remove_link_inner(from, to).await
    }

    async fn remove_note(&mut self, note: &Note) -> Result<()> {
        log::debug!("removing note {:?}", note);

        let mut tx = self.pool.begin().await?;
        Self::remove_note(&mut tx, note).await?;
        tx.commit().await?;

        Ok(())
    }

    async fn rename_note(&mut self, note: &Note, new_name: &str) -> Result<()> {
        log::debug!("renaming note {:?} -> {}", note, new_name);

        let mut tx = self.pool.begin().await?;
        Self::rename_note(&mut tx, note, &new_name).await?;
        tx.commit().await?;

        Ok(())
    }
}
