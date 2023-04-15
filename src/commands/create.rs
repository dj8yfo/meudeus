use syntect::easy::HighlightLines;

use crate::{
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::Note,
    print::format_two_tokens,
};

pub(crate) async fn exec(
    title: &String,
    db: SqliteAsyncHandle,

    is_tag: bool,

    md_static: MarkdownStatic,
) -> Result<String, anyhow::Error> {
    let note = create(title, db, is_tag, md_static).await?;

    Ok(format_two_tokens("note created", &format!("{:?}", note)))
    // Err(anyhow::anyhow!("baby futter"))
}

pub(crate) async fn create(
    title: &String,
    db: SqliteAsyncHandle,

    is_tag: bool,

    md_static: MarkdownStatic,
) -> Result<Note, anyhow::Error> {
    let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
    let note = Note::init(title.clone(), is_tag, &mut highlighter, md_static);

    db.lock().await.save(&note).await?;
    note.persist()?;
    Ok(note)
}
