use std::{collections::HashMap, fmt::Display, sync::Arc};

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{color::ColorScheme, keymap, ExternalCommands},
    highlight::MarkdownStatic,
    link::Link,
    note::Note,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Action {
    Jump(Link),
    Open(Link),
    OpenXDG(Link),
    Return(Note),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Jump(link) => write!(f, "jump : {}", link),
            Self::Open(link) => write!(f, "open : {}", link),
            Self::OpenXDG(link) => write!(f, "open xdg : {}", link),
            Self::Return(note) => write!(f, "return to explore : {}", note),
        }
    }
}
pub(crate) struct Iteration {
    items: Option<Vec<Link>>,
    multi: bool,
    external_commands: ExternalCommands,
    return_note: Note,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::surf::Bindings,
}

impl Iteration {
    pub(crate) fn new(
        items: Vec<Link>,
        multi: bool,
        external_commands: ExternalCommands,
        return_note: Note,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        bindings_map: keymap::surf::Bindings,
    ) -> Self {
        Self {
            items: Some(items),
            multi,
            external_commands,
            return_note,
            md_static,
            color_scheme,
            bindings_map,
        }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Action> {
        let items = self.items.take().unwrap();

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
        let note_hint = self.return_note.name();
        for mut link in items {
            let tx_double = tx.clone();
            let ext_cmds_double = self.external_commands.clone();
            tokio::task::spawn(async move {
                link.prepare_display();
                link.prepare_preview(&ext_cmds_double.preview, self.md_static, self.color_scheme);
                let _result = tx_double.send(Arc::new(link));
                // if result.is_err() {
                //     eprintln!("{}", format!("{:?}", result).red());
                // }
            });
        }

        drop(tx);

        let keys_descriptors = self.bindings_map.keys_descriptors();
        let out = tokio::task::spawn_blocking({
            let mut bindings = vec!["ctrl-c:abort", "ESC:abort", "Enter:accept"];
            bindings.extend(
                keys_descriptors
                    .into_iter()
                    .map(|element| &*(Box::<str>::leak(element.into_boxed_str()))),
            );
            let note_hint = format!("(surf: {}) > ", note_hint);
            move || {
                let options = SkimOptionsBuilder::default()
                    .height(Some("100%"))
                    .preview(Some(""))
                    .prompt(Some(&note_hint))
                    .preview_window(Some("up:50%"))
                    .multi(self.multi)
                    .bind(bindings)
                    .build()
                    .unwrap();
                Skim::run_with(&options, Some(rx))
            }
        })
        .await
        .unwrap();

        let bindings_map: HashMap<tuikit::key::Key, keymap::surf::Action> =
            (&self.bindings_map).into();
        if let Some(out) = out {
            let selected_items = out
                .selected_items
                .iter()
                .map(|selected_item| {
                    (**selected_item)
                        .as_any()
                        .downcast_ref::<Link>()
                        .unwrap()
                        .to_owned()
                })
                .collect::<Vec<Link>>();

            let action = match out.final_key {
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of surf cycle"
                    ))
                }
                Key::Enter => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Action::Open(item.clone()));
                    } else {
                        return Err(anyhow::anyhow!("no item selected"));
                    }
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
                keymap::surf::Action::OpenXDG => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::OpenXDG(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::surf::Action::JumpToLinkOrSnippet => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::Jump(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::surf::Action::ReturnToExplore => Ok(Action::Return(self.return_note)),
            }
        } else {
            Err(anyhow::anyhow!("skim internal errors"))
        }
    }
}
