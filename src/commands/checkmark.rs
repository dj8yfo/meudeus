use futures::future::join_all;
use syntect::easy::HighlightLines;

use crate::{
    config::{color::ColorScheme, keymap, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, NoteTaskItemTerm, PreviewType},
    skim::checkmark::Action as TaskAction,
    skim::checkmark::Iteration as CheckmarkIteration,
    skim::explore::Action,
    task_item::{TaskItem, TaskTreeWrapper},
    Jump, Yank,
};

use super::explore::iteration;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    surf: SurfParsing,
    external_commands: ExternalCommands,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::checkmark::Bindings,
    explore_bindings_map: keymap::explore::Bindings,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list(md_static, color_scheme).await?;

    let nested_threshold = 1;
    let mut preview_type = PreviewType::TaskStructure;
    let note = loop {
        let (next_items, opened, preview_type_after) = iteration(
            db.clone(),
            list,
            &external_commands,
            &surf,
            preview_type,
            md_static,
            color_scheme,
            true,
            nested_threshold,
            explore_bindings_map.clone(),
        )
        .await?;
        preview_type = preview_type_after;
        list = next_items;

        if let Some(Action::Open(opened)) = opened {
            break opened;
        }
    };
    let _note = checkmark_note(note, &external_commands, &surf, md_static, bindings_map).await?;
    Ok("success".to_string())
}

pub(crate) async fn checkmark_note(
    note: Note,
    external_commands: &ExternalCommands,
    surf: &SurfParsing,
    md_static: MarkdownStatic,
    bindings_map: keymap::checkmark::Bindings,
) -> Result<Note, anyhow::Error> {
    let mut next_tasks_window = None;
    let mut tasks = read_tasks_from_file(&note, surf, md_static).await?;
    loop {
        let action = CheckmarkIteration::new(tasks, note.clone(), bindings_map.clone()).run()?;
        next_tasks_window = match action {
            TaskAction::Toggle(selected_tasks) => {
                for task in selected_tasks {
                    task.toggle()?;
                }
                next_tasks_window
            }
            TaskAction::Open(task) => {
                let note_task_item_term = task.data.0.root;
                note_task_item_term.jump(external_commands.open.clone())?;

                None
            }
            TaskAction::Yank(task) => {
                task.yank(external_commands.open.clone())?;
                next_tasks_window
            }
            TaskAction::Widen => None,
            TaskAction::Narrow(start, end) => Some((start, end)),

            TaskAction::Return(note) => {
                return Ok(note);
            }
        };
        tasks = match next_tasks_window {
            None => read_tasks_from_file(&note, surf, md_static).await?,
            Some((start, end)) => {
                let all = read_tasks_from_file(&note, surf, md_static).await?;
                all[start..end].to_vec()
            }
        };
    }
}

async fn read_tasks_from_file(
    note: &Note,
    surf: &SurfParsing,
    md_static: MarkdownStatic,
) -> Result<Vec<TaskTreeWrapper>, anyhow::Error> {
    let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
    let tasks = TaskItem::parse(note, surf, &mut highlighter, md_static)?;

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
    Ok(tasks)
}
