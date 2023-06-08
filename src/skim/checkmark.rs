use std::{collections::HashMap, sync::Arc};

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    config::keymap,
    note::{Note, NoteTaskItemTerm},
    task_item::TaskTreeWrapper,
};

#[allow(clippy::large_enum_variant)]
pub enum Action {
    Toggle(Vec<TaskTreeWrapper>),
    Open(TaskTreeWrapper),
    Yank(TaskTreeWrapper),
    Widen,
    Narrow(usize, usize),
    Return(Note),
}

pub(crate) struct Iteration {
    items: Option<Vec<TaskTreeWrapper>>,
    return_note: Note,
    bindings_map: keymap::checkmark::Bindings,
}
impl Iteration {
    pub(crate) fn new(
        items: Vec<TaskTreeWrapper>,
        return_note: Note,
        bindings_map: keymap::checkmark::Bindings,
    ) -> Self {
        Self {
            items: Some(items),
            return_note,
            bindings_map,
        }
    }

    pub(crate) fn run(mut self) -> anyhow::Result<Action> {
        let keys_descriptors = self.bindings_map.keys_descriptors();
        let mut bindings = vec!["ctrl-c:abort", "ESC:abort", "Enter:accept"];
        bindings.extend(
            keys_descriptors
                .into_iter()
                .map(|element| &*(Box::<str>::leak(element.into_boxed_str()))),
        );
        let items = self.items.take().unwrap();
        let note_hint = format!("(checkmark: {}) > ", self.return_note.name());
        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .prompt(Some(&note_hint))
            .preview_window(Some("right:65%"))
            .multi(true)
            .bind(bindings)
            .build()?;

        let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

        let _jh = tokio::task::spawn_blocking(move || {
            for link in items.into_iter().rev() {
                let _result = tx.send(Arc::new(link));
                // if result.is_err() {
                //     eprintln!("{}", format!("{:?}", result).red());
                // }
            }
        });

        let bindings_map: HashMap<tuikit::key::Key, keymap::checkmark::Action> =
            (&self.bindings_map).into();
        if let Some(out) = Skim::run_with(&options, Some(rx)) {
            let selected_items = out
                .selected_items
                .iter()
                .map(|selected_item| {
                    (**selected_item)
                        .as_any()
                        .downcast_ref::<TaskTreeWrapper>()
                        .unwrap()
                        .clone()
                })
                .collect::<Vec<TaskTreeWrapper>>();

            let action = match out.final_key {
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of checkmark cycle"
                    ))
                }
                Key::Enter => {
                    return Ok(Action::Toggle(selected_items));
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
                keymap::checkmark::Action::JumpToTask => {
                    let first = selected_items.first().expect("non empty");
                    Ok(Action::Open(first.clone()))
                }
                keymap::checkmark::Action::CopyTaskSubtree => {
                    let first = selected_items.first().expect("non empty");
                    Ok(Action::Yank(first.clone()))
                }
                keymap::checkmark::Action::WidenContext => Ok(Action::Widen),
                keymap::checkmark::Action::NarrowContext => {
                    let first = selected_items.first().expect("non empty");
                    let (start, end) = match first.data.0.root {
                        NoteTaskItemTerm::Note(..) => unreachable!("note"),
                        NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
                        NoteTaskItemTerm::TaskHint(..) => unreachable!("hint"),
                        NoteTaskItemTerm::Task(ref task) | NoteTaskItemTerm::TaskMono(ref task) => {
                            let next_index = match task.next_index {
                                Some(next_index) => next_index,
                                None => task.self_index + 1,
                            };
                            (task.self_index, next_index)
                        }
                    };
                    Ok(Action::Narrow(start, end))
                }
                keymap::checkmark::Action::ReturnToExplore => Ok(Action::Return(self.return_note)),
            }
        } else {
            Err(anyhow::anyhow!("skim internal errors"))
        }
    }
}
