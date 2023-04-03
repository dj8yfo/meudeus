use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::SqliteAsyncHandle,
    note::{DynResources, Note, PreviewType},
};

pub(crate) struct Iteration {
    db: SqliteAsyncHandle,
    items: Option<Vec<Note>>,
    multi: bool,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
}

impl Iteration {
    pub(crate) fn new(
        items: Vec<Note>,
        db: SqliteAsyncHandle,
        multi: bool,
        external_commands: ExternalCommands,
        surf_parsing: SurfParsing,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            multi,
            external_commands,
            surf_parsing,
        }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Note> {
        let items = self.items.take().unwrap();

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let db = self.db;
        for mut note in items {
            let db_double = db.clone();
            let ext_double = self.external_commands.clone();
            let surf_parsing = self.surf_parsing.clone();
            let tx_double = tx.clone();

            tokio::task::spawn(async move {
                note.set_resources(DynResources {
                    external_commands: ext_double,
                    surf_parsing,
                    preview_type: PreviewType::Details,
                    preview_result: None,
                });
                note.prepare_preview(&db_double).await;
                let result = tx_double.send(Arc::new(note));
                if result.is_err() {
                    // eprintln!("{}",format!("{:?}", result).red());
                }
            });
        }

        let out = tokio::task::spawn_blocking(move || {
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview(Some(""))
                .prompt(Some("(select) >"))
                .preview_window(Some("right:65%"))
                .multi(self.multi)
                .bind(vec!["ctrl-c:abort", "Enter:accept", "ESC:abort"])
                .build()
                .unwrap();

            Skim::run_with(&options, Some(rx))
        })
        .await
        .unwrap();

        if let Some(out) = out {
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
