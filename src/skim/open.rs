

use colored::Colorize;
use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};
use tokio::runtime::Handle;

use crate::{
    database::SqliteAsyncHandle,
    note::{AsyncQeuryResources, Note},
};

pub(crate) struct Iteration {
    db: SqliteAsyncHandle,
    items: Option<Vec<Note>>,
    multi: bool,
}

impl Iteration {
    pub(crate) fn new(items: Vec<Note>, db: SqliteAsyncHandle, multi: bool) -> Self {
        Self {
            items: Some(items),
            db,
            multi,
        }
    }

    /// This method launches a `skim` fuzzy search with `items` and
    /// returns the selected items as their original type.
    pub(crate) fn run(mut self) -> anyhow::Result<Note> {
        let mut items = self.items.take().unwrap();
        if items.len() == 1 {
            return Ok(items.remove(0));
        }

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .multi(self.multi)
            .bind(vec!["ctrl-c:abort", "Enter:accept", "ESC:abort"])
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

        if let Some(out) = Skim::run_with(&options, Some(rx)) {
            let selected_items = out
                .selected_items
                .iter()
                .map(|selected_item| {
                    (**selected_item)
                        .as_any()
                        .downcast_ref::<Note>()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<Note>>();

            match out.final_key {
                Key::Enter => {
                    if let Some(item) = selected_items.first() {
                        return Ok(item.clone());
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!("user chose to abort infinite cycle"))
                }
                _ => {
                    unreachable!();
                }
            };
        } else {
            return Err(anyhow::anyhow!("skim internal errors"));
        }
    }
}
