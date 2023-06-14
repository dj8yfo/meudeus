use std::{collections::HashMap, rc::Rc, sync::Arc};

use skim::{
    prelude::{unbounded, DefaultSkimSelector, Key, SkimOptionsBuilder},
    Selector, Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::{color::ColorScheme, keymap, ExternalCommands, SurfParsing},
    database::SqliteAsyncHandle,
    highlight::MarkdownStatic,
    note::{DynResources, Note, PreviewType},
};
pub enum Action {
    Select(Note),
    Return(Vec<Note>),
    TogglePreview,
    Pop(Note),
    MoveTopmost(Note),
    SwapWithAbove(Note),
    SwapWithBelow(Note),
}

pub(crate) struct Iteration {
    db: SqliteAsyncHandle,
    input_items_from_explore: Vec<Note>,
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
    bindings_map: keymap::stack::Bindings,
    preselected_item: Option<String>,
}

impl Iteration {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        hint: String,
        input_items_from_explore: Vec<Note>,
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
        bindings_map: keymap::stack::Bindings,
        selected_item: Option<String>,
    ) -> Self {
        Self {
            items: Some(items),
            db,
            input_items_from_explore,
            multi,
            preview_type,
            external_commands,
            surf_parsing,
            hint,
            md_static,
            color_scheme,
            straight,
            nested_threshold,
            bindings_map,
            preselected_item: selected_item,
        }
    }

    pub(crate) async fn run(mut self) -> anyhow::Result<Action> {
        let items = self.items.take().unwrap();

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let db = self.db;
        let db_double = db.clone();
        let ext_double = self.external_commands.clone();
        let surf_parsing = self.surf_parsing.clone();

        tokio::task::spawn(async move {
            for mut note in items {
                let ext_double = ext_double.clone();
                let surf_parsing = surf_parsing.clone();
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
                let result = tx.send(Arc::new(note));
                if result.is_err() {
                    // eprintln!("{}",format!("{:?}", result).red());
                }
            }
        });

        let hint = self.hint;
        let keys_descriptors = self.bindings_map.keys_descriptors();
        let out = tokio::task::spawn_blocking(move || {
            let mut bindings = vec!["ctrl-c:abort", "ESC:abort", "Enter:accept"];
            bindings.extend(
                keys_descriptors
                    .into_iter()
                    .map(|element| &*(Box::<str>::leak(element.into_boxed_str()))),
            );
            let hint = format!("({hint}) > ");
            let selector = self.preselected_item.map(|item| {
                let preset_items = vec![item];
                let selector = DefaultSkimSelector::default().preset(preset_items);
                let selector: Rc<dyn Selector> = Rc::new(selector);
                selector
            });
            let options = SkimOptionsBuilder::default()
                .height(Some("100%"))
                .preview(Some(""))
                .prompt(Some(&hint))
                .preview_window(Some("up:60%"))
                .multi(self.multi)
                .bind(bindings)
                .selector(selector)
                .build()
                .unwrap();

            Skim::run_with(&options, Some(rx))
        })
        .await
        .unwrap();

        let bindings_map: HashMap<tuikit::key::Key, keymap::stack::Action> =
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
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of open cycle"
                    ))
                }
                Key::Enter => {
                    if let Some(item) = selected_items.first() {
                        return Ok(Action::Select(item.clone()));
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
                keymap::stack::Action::ReturnToExplore => {
                    if let Some(_item) = selected_items.first() {
                        Ok(Action::Return(self.input_items_from_explore))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::TogglePreviewType => {
                    if let Some(_item) = selected_items.first() {
                        Ok(Action::TogglePreview)
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::PopNoteFromStack => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::Pop(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::MoveNoteToStackTop => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::MoveTopmost(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::SwapWithAbove => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::SwapWithAbove(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::SwapWithBelow => {
                    if let Some(item) = selected_items.first() {
                        Ok(Action::SwapWithBelow(item.clone()))
                    } else {
                        Err(anyhow::anyhow!("no item selected"))
                    }
                }
                keymap::stack::Action::DeselectAll => {
                    unreachable!("deselect_all must be unreachable");
                }
            }
        } else {
            Err(anyhow::anyhow!("skim internal errors"))
        }
    }
}
