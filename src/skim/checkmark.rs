use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{
    note::{Note, NoteTaskItemTerm},
    task_item::TaskTreeWrapper,
};

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
}
impl Iteration {
    pub(crate) fn new(items: Vec<TaskTreeWrapper>, return_note: Note) -> Self {
        Self {
            items: Some(items),
            return_note,
        }
    }

    pub(crate) fn run(mut self) -> anyhow::Result<Action> {
        let items = self.items.take().unwrap();
        let note_hint = format!("(checkmark: {}) > ", self.return_note.name());
        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .prompt(Some(&note_hint))
            .preview_window(Some("right:65%"))
            .multi(true)
            .bind(vec![
                "ctrl-c:abort",
                "Enter:accept",
                "ESC:abort",
                "ctrl-j:accept",
                "ctrl-y:accept",
                "ctrl-w:accept",
                "ctrl-l:accept",
                "ctrl-e:abort",
            ])
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

            match out.final_key {
                Key::Enter => {
                    return Ok(Action::Toggle(selected_items));
                }
                Key::Ctrl('j') => {
                    let first = selected_items.first().expect("non empty");
                    return Ok(Action::Open(first.clone()));
                }
                Key::Ctrl('y') => {
                    let first = selected_items.first().expect("non empty");
                    return Ok(Action::Yank(first.clone()));
                }
                Key::Ctrl('w') => {
                    return Ok(Action::Widen);
                }
                Key::Ctrl('l') => {
                    let first = selected_items.first().expect("non empty");
                    let (start, end) = match first.data.0.root {
                        NoteTaskItemTerm::Note(..) => unreachable!("note"),
                        NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
                        NoteTaskItemTerm::TaskHint(..) => unreachable!("hint"),
                        NoteTaskItemTerm::Task(ref task) | NoteTaskItemTerm::TaskMono(ref task) => {
                            (task.self_index, task.next_index.unwrap())
                        }
                    };
                    return Ok(Action::Narrow(start, end));
                }
                Key::Ctrl('c') | Key::ESC => {
                    return Err(anyhow::anyhow!(
                        "user chose to abort current iteration of checkmark cycle"
                    ))
                }
                Key::Ctrl('e') => {
                    return Ok(Action::Return(self.return_note));
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
