use colored::Colorize;
use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::ExternalCommands,
    database::SqliteAsyncHandle,
    note::{AsyncQeuryResources, Note},
};

pub(crate) struct Iteration {
    db: SqliteAsyncHandle,
    items: Option<Vec<Note>>,
    multi: bool,
    external_commands: ExternalCommands,
}

impl Iteration {
    pub(crate) fn new(
        items: Vec<Note>,
        db: SqliteAsyncHandle,
        multi: bool,
        external_commands: ExternalCommands,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            multi,
            external_commands,
        }
    }

    pub(crate) fn run(mut self) -> anyhow::Result<Note> {
        let items = self.items.take().unwrap();

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .multi(self.multi)
            .bind(vec!["ctrl-c:abort", "Enter:accept", "ESC:abort"])
            .build()?;

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let db = self.db;
        let _jh = std::thread::spawn(move || {
            for mut note in items {
                note.set_resources(AsyncQeuryResources {
                    db: db.clone(),
                    file_preview_cmd: self.external_commands.preview.file_cmd.clone(),
                });
                let result = tx.send(Arc::new(note));
                if result.is_err() {
                    eprintln!("{}", format!("{:?}", result).red());
                }
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
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of open cycle"
                    ))
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
