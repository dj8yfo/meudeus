use std::time::Duration;

use tokio::time::sleep;

use crate::{
    config::{color::ColorScheme, keymap, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, PreviewType},
    print::format_two_tokens,
    skim::stack_sequential::Action,
};

use super::explore::GLOBAL_STACK;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    input_items_from_explore: Vec<Note>,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::stack::Bindings,
) -> Result<String, anyhow::Error> {
    let notes = stack_select(
        db,
        input_items_from_explore,
        external_commands,
        surf_parsing,
        md_static,
        color_scheme,
        bindings_map,
    )
    .await?;
    Ok(notes
        .first()
        .map(|note| note.name())
        .unwrap_or("".to_string()))
}

pub(crate) async fn stack_select(
    db: SqliteAsyncHandle,
    input_items_from_explore: Vec<Note>,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::stack::Bindings,
) -> Result<Vec<Note>, anyhow::Error> {
    let straight = true;
    let multi = true;
    let nested_threshold = 1;
    let mut preview_type = PreviewType::TaskStructure;
    let mut preselected_item = None;
    loop {
        let list = db
            .lock()
            .await
            .select_from_stack(GLOBAL_STACK, md_static, color_scheme)
            .await?;
        let action = crate::skim::stack_sequential::Iteration::new(
            format!("stack; {GLOBAL_STACK}"),
            input_items_from_explore.clone(),
            list,
            db.clone(),
            multi,
            preview_type,
            external_commands.clone(),
            surf_parsing.clone(),
            md_static,
            color_scheme,
            straight,
            nested_threshold,
            bindings_map.clone(),
            preselected_item.clone(),
        )
        .run()
        .await?;

        match action {
            Action::Select(notes) => {
                println!(
                    "{}",
                    format_two_tokens(
                        "selected ",
                        &notes
                            .iter()
                            .map(|note| note.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                );
                return Ok(notes);
            }
            Action::Return(notes) => {
                println!(
                    "{}",
                    format_two_tokens("returning to previous", "selection in explore")
                );
                return Ok(notes);
            }
            Action::TogglePreview => {
                preselected_item = None;
                preview_type = preview_type.toggle();
            }
            Action::Pop(note) => {
                preselected_item = None;
                let name = note.name();
                db.lock()
                    .await
                    .pop_note_from_stack(GLOBAL_STACK, &name)
                    .await?;
                println!(
                    "{}",
                    format_two_tokens("popped ", &format!("{name} from {GLOBAL_STACK}"))
                );
                sleep(Duration::new(0, 500_000_000)).await;
            }
            Action::MoveTopmost(note) => {
                preselected_item = None;
                let name = note.name();
                db.lock().await.move_to_topmost(GLOBAL_STACK, &name).await?;
                println!(
                    "{}",
                    format_two_tokens("moved to topmost ", &format!("{name} in {GLOBAL_STACK}"))
                );
                sleep(Duration::new(0, 500_000_000)).await;
            }
            Action::SwapWithAbove(note) => {
                let name = note.name();
                db.lock().await.swap_with_above(GLOBAL_STACK, &name).await?;
                preselected_item = Some(note.name());
                println!(
                    "{}",
                    format_two_tokens("swapped with above ", &format!("{name} in {GLOBAL_STACK}"))
                );
            }
            Action::SwapWithBelow(note) => {
                let name = note.name();
                db.lock().await.swap_with_below(GLOBAL_STACK, &name).await?;
                preselected_item = Some(note.name());
                println!(
                    "{}",
                    format_two_tokens("swapped with below ", &format!("{name} in {GLOBAL_STACK}"))
                );
            }
        }
    }
}
