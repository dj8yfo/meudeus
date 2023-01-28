use crate::{
    database::{Database, SqliteAsyncHandle},
    note::Note,
    print::print_two_tokens,
    Directory,
};

pub(crate) async fn exec(
    title: &String,
    dir: Directory,
    db: SqliteAsyncHandle,

    is_tag: bool,
) -> Result<String, anyhow::Error> {
    dir.check()?;
    let note = Note::init(title.clone(), dir, is_tag);

    db.lock().await.save(&note).await?;
    note.persist()?;

    Ok(print_two_tokens("note created", &format!("{:?}", note)))
    // Err(anyhow::anyhow!("baby futter"))
}
