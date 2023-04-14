use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::{note::NoteTaskItemTerm, task_item::TaskTreeWrapper};

pub enum Action {
    Toggle(Vec<TaskTreeWrapper>),
    Open(TaskTreeWrapper),
    Yank(TaskTreeWrapper),
    Widen,
    Narrow(usize, usize),
}

pub(crate) struct Iteration {
    items: Option<Vec<TaskTreeWrapper>>,
}
impl Iteration {
    pub(crate) fn new(items: Vec<TaskTreeWrapper>) -> Self {
        Self { items: Some(items) }
    }

    pub(crate) fn run(mut self) -> anyhow::Result<Action> {
        let items = self.items.take().unwrap();

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .prompt(Some("(checkmark) > "))
            .preview_window(Some("up:80%"))
            .multi(true)
            .bind(vec![
                "ctrl-c:abort",
                "Enter:accept",
                "ESC:abort",
                "ctrl-j:accept",
                "ctrl-y:accept",
                "ctrl-w:accept",
                "ctrl-l:accept",
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
                _ => {
                    unreachable!();
                }
            };
        } else {
            return Err(anyhow::anyhow!("skim internal errors"));
        }
    }
}
