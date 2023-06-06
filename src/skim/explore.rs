use std::{collections::HashMap, sync::Arc};

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{color::ColorScheme, keymap, ExternalCommands, SurfParsing},
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
    bindings_map: keymap::explore::Bindings,
}

pub enum Action {
    Open(Note),
    OpenXDG(Note),
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
        bindings_map: keymap::explore::Bindings,
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
            bindings_map,
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
        let keys_descriptors = self.bindings_map.keys_descriptors();
        let out = tokio::task::spawn_blocking(move || {
            let mut bindings = vec!["ctrl-c:abort", "ESC:abort", "Enter:accept"];
            bindings.extend(
                keys_descriptors
                    .into_iter()
                    .map(|element| &*(Box::<str>::leak(element.into_boxed_str()))),
            );
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview_window(Some("up:70%"))
                .preview(Some(""))
                .prompt(Some(&hint))
                .multi(true)
                .bind(bindings)
                .build()
                .unwrap();
            Skim::run_with(&options, Some(rx))
        })
        .await
        .unwrap();

        let bindings_map: HashMap<tuikit::key::Key, keymap::explore::Action> =
            (&self.bindings_map).into();
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

            let action = match out.final_key {
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
                key @ Key::Ctrl(..) | key @ Key::Alt(..) => bindings_map.get(&key).cloned(),
                _ => {
                    unreachable!();
                }
            };
            let Some(action) = action else {
                unreachable!("an unspecified keybinding isn't expected to pick None from Hashmap<Key, Action>");
            };
            match action {
                keymap::explore::Action::OpenXDG => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::OpenXDG(item.clone()),
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::PopulateSearchWithBacklinks => {
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

                keymap::explore::Action::PopulateSearchWithForwardlinks => {
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

                keymap::explore::Action::TogglePreviewType => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::TogglePreview,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::WidenToAllNotes => {
                    return Ok(Out {
                        action: Action::Widen,
                        next_items: vec![],
                    });
                }

                keymap::explore::Action::RenameNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Rename(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::LinkFromSelectedNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Link(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::UnlinkFromSelectedNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Unlink(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::RemoveNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Remove(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::CreateAutolinkedNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::CreateLinkedFrom(item.clone()),
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::SurfNoteSubtree => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Surf(item.clone()),
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::ToggleLinksDirection => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::InvertLinks,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::SpliceReachableChildrenOfNote => {
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

                keymap::explore::Action::NarrowSelection => {
                    return Ok(Out {
                        action: Action::Narrow,
                        next_items: selected_items,
                    });
                }

                keymap::explore::Action::IncreaseUnlistedThreshold => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::IncreaseUnlistedThreshold,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::DccreaseUnlistedThreshold => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::DecreaseUnlistedThreshold,
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }

                keymap::explore::Action::PushNoteToStack => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::PushToStack(item.clone()),
                            next_items: items,
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                keymap::explore::Action::SwitchModeToStack => {
                    if let Some(_item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::SwitchToStack,
                            next_items: vec![],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
                keymap::explore::Action::CheckmarkNote => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Out {
                            action: Action::Checkmark(item.clone()),
                            next_items: vec![item.clone()],
                        });
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
                }
            }
        } else {
            return Err(anyhow::anyhow!("skim internal errors"));
        }
    }
}
