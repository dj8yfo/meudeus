use std::time::Duration;

use tokio::time::sleep;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing, keymap},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, PreviewType},
    print::format_two_tokens,
    skim::stack_sequential::Action,
};

use super::explore::GLOBAL_STACK;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::stack::Bindings,
) -> Result<String, anyhow::Error> {
    let note = stack_select(db, external_commands, surf_parsing, md_static, color_scheme, bindings_map).await?;
    Ok(note.name())
}


pub(crate) async fn stack_select(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::stack::Bindings,
) -> Result<Note, anyhow::Error> {
    let straight = true;
    let multi = false;
    let nested_threshold = 1;
    let mut preview_type = PreviewType::TaskStructure;
    loop {
        let list = db
            .lock()
            .await
            .select_from_stack(GLOBAL_STACK, md_static, color_scheme)
            .await?;
        let action = crate::skim::stack_sequential::Iteration::new(
            format!("stack; {GLOBAL_STACK}"),
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
        )
        .run()
        .await?;

        match action {
            Action::Select(note) => {
                println!("{}", format_two_tokens("selected ", &note.name()));
                return Ok(note);
            }
            Action::TogglePreview => {
                preview_type = preview_type.toggle();
            }
            Action::Pop(note) => {
                let name = note.name();
                db.lock()
                    .await
                    .pop_note_from_stack(GLOBAL_STACK, &name)
                    .await?;
                println!(
                    "{}",
                    format_two_tokens("popped ", &format!("{name} from {GLOBAL_STACK}"))
                );
                sleep(Duration::new(1, 0)).await;
            }
            Action::MoveTopmost(note) => {
                let name = note.name();
                db.lock().await.move_to_topmost(GLOBAL_STACK, &name).await?;
                println!(
                    "{}",
                    format_two_tokens("moved to topmost ", &format!("{name} in {GLOBAL_STACK}"))
                );
                sleep(Duration::new(1, 0)).await;
            }
        }
    }
}
