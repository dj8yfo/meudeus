use crate::{
    database::{Database, SqliteAsyncHandle},
    note::Note,
    print::print_two_tokens,
};

pub(crate) async fn exec(
    title: &String,
    db: SqliteAsyncHandle,

    is_tag: bool,
) -> Result<String, anyhow::Error> {
    let note = Note::init(title.clone(), is_tag);

    db.lock().await.save(&note).await?;
    note.persist()?;

    Ok(print_two_tokens("note created", &format!("{:?}", note)))
    // Err(anyhow::anyhow!("baby futter"))
}
