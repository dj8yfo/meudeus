use futures::future::join_all;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    note::{NoteTaskItemTerm, PreviewType},
    skim::checkmark::Action as TaskAction,
    skim::checkmark::Iteration as CheckmarkIteration,
    skim::explore::{Action, Iteration},
    task_item::{TaskItem, TaskTreeWrapper},
    Jump, Yank,
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

        let tasks_stereo = NoteTaskItemTerm::parse(&tasks, false, false);
        let tasks_mono = NoteTaskItemTerm::parse(&tasks, false, true);
        let tasks = tasks_stereo
            .into_iter()
            .zip(tasks_mono.into_iter())
            .collect::<Vec<_>>();

        let compute_display_jh = tasks
            .into_iter()
            .map(|element| {
                tokio::task::spawn(async move {
                    let mut wrapper = TaskTreeWrapper {
                        data: element,
                        display_item: None,
                        preview_item: None,
                        mono_preview_item: None,
                    };

                    wrapper.prepare_display();
                    wrapper.prepare_preview();
                    wrapper
                })
            })
            .collect::<Vec<_>>();
        let tasks = join_all(compute_display_jh).await;
        let tasks = tasks
            .into_iter()
            .map(|result| result.expect("we do not expect preview generation to panic"))
            .collect::<Vec<_>>();

        let action = CheckmarkIteration::new(tasks).run()?;
        match action {
            TaskAction::Toggle(selected_tasks) => {
                for task in selected_tasks {
                    task.toggle()?;
                }
            }
            TaskAction::Open(task) => {
                let note_task_item_term = task.data.0.root;
                note_task_item_term.jump(external_commands.open.clone())?;
            }
            TaskAction::Yank(task) => {
                task.yank(external_commands.open.clone())?;
            }
        }
    }
}
