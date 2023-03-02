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
    external_commands: ExternalCommands,
}

pub enum Action {
    Open(Note),
    Noop,
}

pub struct Out {
    pub action: Action,
    pub next_items: Vec<Note>,
}

impl Iteration {
    pub(crate) fn new(
        items: Vec<Note>,
        db: SqliteAsyncHandle,
        external_commands: ExternalCommands,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            external_commands,
        }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Out> {
        let items = self.items.take().unwrap();

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .multi(false)
            .bind(vec![
                "ctrl-c:abort",
                "Enter:accept",
                "ESC:abort",
                "ctrl-h:accept",
                "ctrl-l:accept",
            ])
            .build()?;

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let db = self.db;
        let cloned = items.clone();
        let _jh = std::thread::spawn(move || {
            for mut note in cloned {
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
                        return Ok(Out {
                            action: Action::Open(item.clone()),
                            next_items: items,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of explore cycle"
                    ))
                }

                Key::Ctrl('h') => {
                    if let Some(item) = selected_items.first() {
                        let mut next = item.fetch_backlinks().await.unwrap()?;
                        if next.is_empty() {
                            next = items;
                        }
                        return Ok(Out {
                            action: Action::Noop,
                            next_items: next,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('l') => {
                    if let Some(item) = selected_items.first() {
                        let mut next = item.fetch_forward_links().await.unwrap()?;
                        if next.is_empty() {
                            next = items;
                        }
                        return Ok(Out {
                            action: Action::Noop,
                            next_items: next,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
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
