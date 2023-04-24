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
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    preview_type: PreviewType,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    straight: bool,
    nested_threshold: usize,
}

pub enum Action {
    Open(Note),
    Link(Note),
    Unlink(Note),
    Back,
    Forward,
    Widen,
    Rename(Note),
    Remove(Note),
    CreateLinkedFrom(Note),
    Surf(Note),
    Checkmark(Note),
    TogglePreview,
    InvertLinks,
    Splice,
    Narrow,
    IncreaseUnlistedThreshold,
    DecreaseUnlistedThreshold,
    PushToStack(Note),
    SwitchToStack,
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
        surf_parsing: SurfParsing,
        preview_type: PreviewType,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
        nested_threshold: usize,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            external_commands,
            surf_parsing,
            preview_type,
            md_static,
            color_scheme,
            straight,
            nested_threshold,
        }
    }
    pub(crate) async fn run(mut self) -> anyhow::Result<Out> {
        let items = self.items.take().unwrap();

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let db = self.db.clone();
        let cloned = items.clone();

        for mut note in cloned {
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
                    // eat up errors on receiver closed
                    // eprintln!("{}", format!("very bad {:?}", result).red());
                }
            });
        }
        drop(tx);

        let dir = if self.straight { "forward" } else { "backward" };
        let hint = format!("(explore; {}) > ", dir);
        let out = tokio::task::spawn_blocking(move || {
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview_window(Some("up:70%"))
                .preview(Some(""))
                .prompt(Some(&hint))
                .multi(true)
                .bind(vec![
                    "ctrl-c:abort",
                    "Enter:accept",
                    "ESC:abort",
                    "ctrl-h:accept",
                    "ctrl-l:accept",
                    "ctrl-t:accept",
                    "ctrl-w:accept",
                    "ctrl-s:accept",
                    "ctrl-k:accept",
                    "alt-r:accept",
                    "alt-l:accept",
                    "alt-u:accept",
                    "alt-d:accept",
                    "alt-c:accept",
                    "alt-f:accept",
                    "alt-s:accept",
                    "alt-n:accept",
                    "alt-o:accept",
                    "alt-p:accept",
                    "alt-a:accept",
                    "ctrl-a:accept",
                ])
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
                        return Ok(Out {
                            action: Action::Open(item.clone()),
                            next_items: vec![item.clone()],
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
                        let mut next = item
                            .fetch_backlinks(
                                &self.db,
                                self.md_static,
                                self.color_scheme,
                                self.straight,
                            )
                            .await?;
                        if next.is_empty() {
                            next = items;
                        }
                        return Ok(Out {
                            action: Action::Back,
                            next_items: next,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('l') => {
                    if let Some(item) = selected_items.first() {
                        let mut next = item
                            .fetch_forward_links(
                                &self.db,
                                self.md_static,
                                self.color_scheme,
                                self.straight,
                            )
                            .await?;
                        if next.is_empty() {
                            next = vec![item.clone()];
                        }
                        return Ok(Out {
                            action: Action::Forward,
                            next_items: next,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('t') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::TogglePreview,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('w') => {
                    return Ok(Out {
                        action: Action::Widen,
                        next_items: vec![],
                    });
                }

                Key::Alt('r') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Rename(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('l') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Link(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('u') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Unlink(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('d') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Remove(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('c') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::CreateLinkedFrom(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Ctrl('s') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Surf(item.clone()),
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('f') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::InvertLinks,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('s') => {
                    if let Some(item) = selected_items.first() {
                        let next = item
                            .reachable_notes(
                                db,
                                self.md_static,
                                self.color_scheme,
                                self.straight,
                                false,
                            )
                            .await?;
                        return Ok(Out {
                            action: Action::Splice,
                            next_items: next,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('n') => {
                    return Ok(Out {
                        action: Action::Narrow,
                        next_items: selected_items,
                    });
                }

                Key::Alt('p') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::IncreaseUnlistedThreshold,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('o') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::DecreaseUnlistedThreshold,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                Key::Alt('a') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::PushToStack(item.clone()),
                            next_items: items,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                Key::Ctrl('a') => {
                    if let Some(_item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::SwitchToStack,
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                Key::Ctrl('k') => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Checkmark(item.clone()),
                            next_items: vec![item.clone()],
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
