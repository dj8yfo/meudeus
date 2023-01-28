use colored::Colorize;
use std::sync::Arc;

use skim::{
    prelude::{unbounded, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};
use tokio::runtime::Handle;

use crate::{
    database::SqliteAsyncHandle,
    note::{AsyncQeuryResources, Note},
};

pub(crate) struct Search {
    db: SqliteAsyncHandle,
    items: Option<Vec<Note>>,
    multi: bool,
}

impl Search {
    pub(crate) fn new(items: Vec<Note>, db: SqliteAsyncHandle, multi: bool) -> Self {
        Self {
            items: Some(items),
            db,
            multi,
        }
    }

    /// This method launches a `skim` fuzzy search with `items` and
    /// returns the selected items as their original type.
    pub(crate) fn run(mut self) -> anyhow::Result<Vec<Note>> {
        let items = self.items.take().unwrap();
        if items.len() == 1 {
            return Ok(items);
        }

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .multi(self.multi)
            .build()?;

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let handle = Handle::current();

        let db = self.db;
        let _jh = std::thread::spawn(move || {
            for mut note in items {
                let tx_clone = tx.clone();
                note.set_resources(AsyncQeuryResources { db: db.clone() });

                handle.spawn(async move {
                    let result = tx_clone.send(Arc::new(note));
                    if result.is_err() {
                        println!("{}", format!("{:?}", result).red());
                    }
                });
            }
        });

        let selected_items = Skim::run_with(&options, Some(rx))
            .map(|out| out.selected_items)
            .unwrap_or_else(Vec::new)
            .iter()
            .map(|selected_item| {
                (**selected_item)
                    .as_any()
                    .downcast_ref::<Note>()
                    .unwrap()
                    .to_owned()
            })
            .collect::<Vec<Note>>();

        match selected_items.len() {
            0 => Err(anyhow::anyhow!("no item selected")),
            _ => Ok(selected_items),
        }
    }
}
