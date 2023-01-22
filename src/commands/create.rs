use clap::ArgMatches;

use crate::{
    database::{Database, Sqlite},
    note::Note,
    Directory,
};

pub(crate) async fn exec(
    dir: Directory,
    matches: &ArgMatches,
    mut db: Sqlite,
) -> Result<String, anyhow::Error> {
    dir.check()?;
    let title = matches
        .get_one::<String>("title")
        .ok_or(anyhow::anyhow!("empty title"))?;
    let is_tag = matches.get_flag("tag");
    let note = Note::init(title.clone(), dir, is_tag);

    db.save(&note).await?;
    note.persist()?;

    Ok(format!("note created \"{:?}\"", note))
    // Err(anyhow::anyhow!("baby futter"))
}
