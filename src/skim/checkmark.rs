use std::sync::Arc;

use skim::{
    prelude::{unbounded, Key, SkimOptionsBuilder},
    Skim, SkimItemReceiver, SkimItemSender,
};

use crate::task_item::TaskTreeWrapper;

pub(crate) struct Iteration {
    items: Option<Vec<TaskTreeWrapper>>,
}
impl Iteration {
    pub(crate) fn new(items: Vec<TaskTreeWrapper>) -> Self {
        Self { items: Some(items) }
    }

    pub(crate) fn run(mut self) -> anyhow::Result<Vec<TaskTreeWrapper>> {
        let items = self.items.take().unwrap();

        let options = SkimOptionsBuilder::default()
            .height(Some("100%"))
            .preview(Some(""))
            .preview_window(Some("right:65%"))
            .multi(true)
            .bind(vec!["ctrl-c:abort", "Enter:accept", "ESC:abort"])
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
                    return Ok(selected_items);
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
