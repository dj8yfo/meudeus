use std::sync::{Arc, Mutex};

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    note::{NoteTaskItemTerm, PreviewType},
    skim::checkmark::Iteration as CheckmarkIteration,
    skim::explore::{Action, Iteration},
    task_item::{TaskItem, TaskTreeWrapper},
};

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    surf: SurfParsing,
    external_commands: ExternalCommands,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list().await?;

    let mut preview_type = PreviewType::TaskStructure;
    let note = loop {
        let out = Iteration::new(
            list.clone(),
            db.clone(),
            external_commands.clone(),
            surf.clone(),
            preview_type,
        )
        .run()
        .await?;

        match out.action {
            Action::Noop => {}
            Action::Open(note) => {
                break note;
            }
            Action::TogglePreview => {
                preview_type = preview_type.toggle();
            }
        }
        list = out.next_items;
    };
    loop {
        let tasks = TaskItem::parse(&note, &surf)?;

        let tasks = NoteTaskItemTerm::parse(&tasks, false);
        let tasks = tasks
            .into_iter()
            .map(|el| TaskTreeWrapper(el, Arc::new(Mutex::new(None)), Arc::new(Mutex::new(None))))
            .collect::<Vec<_>>();

        let selected_tasks = CheckmarkIteration::new(tasks).run()?;
        for task in selected_tasks {
            task.toggle()?;
        }
    }
}
