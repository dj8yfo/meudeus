use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::SqliteAsyncHandle,
    highlight::MarkdownStatic,
    note::{DynResources, Note, PreviewType},
};

pub(crate) struct Iteration {
    db: SqliteAsyncHandle,
    items: Option<Vec<Note>>,
    multi: bool,
    preview_type: PreviewType,
    hint: String,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,

    color_scheme: ColorScheme,
    straight: bool,
    nested_threshold: usize,
}

impl Iteration {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        hint: String,
        items: Vec<Note>,
        db: SqliteAsyncHandle,
        multi: bool,
        preview_type: PreviewType,
        external_commands: ExternalCommands,
        surf_parsing: SurfParsing,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
        nested_threshold: usize,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            multi,
            preview_type,
            external_commands,
            surf_parsing,
            hint,
            md_static,
            color_scheme,
            straight,
            nested_threshold,
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
                    preview_type: self.preview_type,
                    preview_result: None,
                });
                note.prepare_preview(
                    &db_double,
                    self.md_static,
                    self.color_scheme,
                    self.straight,
                    self.nested_threshold,
                )
                .await;
                let result = tx_double.send(Arc::new(note));
                if result.is_err() {
                    // eprintln!("{}",format!("{:?}", result).red());
                }
            });
        }
        drop(tx);

        let hint = self.hint;
        let out = tokio::task::spawn_blocking(move || {
            let hint = format!("({hint}) > ");
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview(Some(""))
                .prompt(Some(&hint))
                .preview_window(Some("up:80%"))
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
                        Ok(item.clone())
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                Key::Ctrl('c') | Key::ESC => Err(anyhow::anyhow!(
                    "user chose to abort current iteration of open cycle"
                )),
                _ => {
                    unreachable!();
                }
            }
        } else {
            Err(anyhow::anyhow!("skim internal errors"))
        }
    }
}
