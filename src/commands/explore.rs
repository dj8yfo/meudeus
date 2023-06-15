use std::time::Duration;

use crate::{
    commands::link::{link, link_noninteractive},
    config::{color::ColorScheme, keymap, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::{Note, PreviewType},
    print::format_two_tokens,
    skim::explore::{Action, Iteration},
    Open,
};

use super::{
    checkmark::checkmark_note, create, remove::remove, rename::rename, stack::stack_select,
    surf::surf_note, unlink::unlink,
};
use inquire::Select;
use inquire::Text;
use tokio::time::sleep;

pub static GLOBAL_STACK: &str = "GLOBAL";

#[allow(clippy::too_many_arguments)]
pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    bindings_map: keymap::surf::Bindings,
    chck_bindings_map: keymap::checkmark::Bindings,
    stack_bindings_map: keymap::stack::Bindings,
    explore_bindings_map: keymap::explore::Bindings,
) -> Result<String, anyhow::Error> {
    let mut list = db.lock().await.list(md_static, color_scheme).await?;
    let mut straight = true;

    let mut preview_type = PreviewType::default();

    let mut nested_threshold = 1;
    loop {
        let (next_items, opened, preview_type_after) = iteration(
            db.clone(),
            list,
            &external_commands,
            &surf_parsing,
            preview_type,
            md_static,
            color_scheme,
            straight,
            nested_threshold,
            explore_bindings_map.clone(),
        )
        .await?;
        preview_type = preview_type_after;
        list = next_items;

        match opened {
            Some(Action::Open(opened)) => {
                opened.open(external_commands.open.clone())?;
                eprintln!("{}", format_two_tokens("viewed", &opened.name()));
            }
            Some(Action::OpenXDG(opened)) => {
                opened.open_xdg()?;
                eprintln!("{}", format_two_tokens("viewed xdg", &opened.name()));
            }
            Some(Action::Surf(surfed)) => {
                if let Err(err) = surf_note(
                    surfed,
                    db.clone(),
                    &external_commands,
                    &surf_parsing,
                    md_static,
                    color_scheme,
                    straight,
                    bindings_map.clone(),
                )
                .await
                {
                    eprintln!("surf error: {:?}", err);
                }
            }

            Some(Action::Checkmark(surfed)) => {
                if let Err(err) = checkmark_note(
                    surfed,
                    &external_commands,
                    &surf_parsing,
                    md_static,
                    chck_bindings_map.clone(),
                )
                .await
                {
                    eprintln!("checkmark error: {:?}", err);
                }
            }
            Some(Action::Rename(opened)) => {
                let note = rename(opened, db.clone(), md_static).await?;
                list = vec![note];
            }

            Some(Action::Link(linked_from)) => {
                if let Err(err) = link(
                    linked_from.clone(),
                    db.clone(),
                    &external_commands,
                    &surf_parsing,
                    md_static,
                    color_scheme,
                    straight,
                    nested_threshold,
                )
                .await
                {
                    eprintln!("link error: {:?}", err);
                }
                // list = vec![linked_from];
            }

            Some(Action::Unlink(unlinked_from)) => {
                if let Err(err) = unlink(
                    unlinked_from.clone(),
                    db.clone(),
                    &external_commands,
                    &surf_parsing,
                    md_static,
                    color_scheme,
                    straight,
                )
                .await
                {
                    eprintln!("unlink error: {:?}", err);
                }
                // list = vec![unlinked_from];
            }

            Some(Action::Remove(removed)) => {
                let next = match remove(db.clone(), removed.clone(), true).await {
                    Ok(true) => vec![],
                    Ok(false) => vec![removed],
                    Err(err) => {
                        eprintln!("remove error: {:?}", err);
                        vec![removed]
                    }
                };
                list = next;
            }

            Some(Action::CreateLinkedFrom(linked_from)) => {
                let options: Vec<&str> = vec!["tag", "note"];
                let note_type = Select::new("select note type", options).prompt()?;
                let is_tag = note_type == "tag";

                let new_name = Text::new("Enter name of a new note:").prompt()?;
                let to =
                    create::create(&new_name, db.clone(), is_tag, md_static, color_scheme).await?;

                link_noninteractive(linked_from.clone(), to, db.clone(), straight).await?;
                // list = vec![linked_from];
            }

            Some(Action::InvertLinks) => {
                straight = !straight;
            }

            Some(Action::IncreaseUnlistedThreshold) => {
                nested_threshold += 1;
            }

            Some(Action::DecreaseUnlistedThreshold) => {
                nested_threshold = nested_threshold.saturating_sub(1);
            }

            Some(Action::PushToStack(note)) => {
                let name = &note.name();
                if let Err(err) = db
                    .lock()
                    .await
                    .push_note_to_stack(GLOBAL_STACK, &note.name())
                    .await
                {
                    eprintln!("push to stack error: {:?}", err);
                } else {
                    println!(
                        "{}",
                        format_two_tokens("pushed ", &format!("{name} to {GLOBAL_STACK}"))
                    );
                }
                sleep(Duration::new(1, 0)).await;
            }
            Some(Action::SwitchToStack) => {
                let next = stack_select(
                    db.clone(),
                    list.clone(),
                    external_commands.clone(),
                    surf_parsing.clone(),
                    md_static,
                    color_scheme,
                    stack_bindings_map.clone(),
                )
                .await?;
                list = next;
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn iteration(
    db: SqliteAsyncHandle,
    list: Vec<Note>,
    external_commands: &ExternalCommands,
    surf_parsing: &SurfParsing,
    preview_type: PreviewType,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
    straight: bool,
    nested_threshold: usize,
    bindings_map: keymap::explore::Bindings,
) -> Result<(Vec<Note>, Option<Action>, PreviewType), anyhow::Error> {
    let out = Iteration::new(
        list.clone(),
        db.clone(),
        external_commands.clone(),
        surf_parsing.clone(),
        preview_type,
        md_static,
        color_scheme,
        straight,
        nested_threshold,
        bindings_map,
    )
    .run()
    .await?;

    let res = match out.action {
        Action::Back | Action::Forward => (out.next_items, None, preview_type),
        Action::Widen => (
            db.lock().await.list(md_static, color_scheme).await?,
            None,
            preview_type,
        ),
        action @ Action::Open(..) => (out.next_items, Some(action), preview_type),
        action @ Action::OpenXDG(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Rename(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Link(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Unlink(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Remove(..) => (out.next_items, Some(action), preview_type),
        action @ Action::CreateLinkedFrom(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Surf(..) => (out.next_items, Some(action), preview_type),
        action @ Action::Checkmark(..) => (out.next_items, Some(action), preview_type),
        action @ Action::InvertLinks => (out.next_items, Some(action), preview_type),
        action @ Action::Splice => (out.next_items, Some(action), preview_type),
        action @ Action::Narrow => (out.next_items, Some(action), preview_type),
        action @ Action::IncreaseUnlistedThreshold => (out.next_items, Some(action), preview_type),
        action @ Action::DecreaseUnlistedThreshold => (out.next_items, Some(action), preview_type),
        action @ Action::PushToStack(..) => (out.next_items, Some(action), preview_type),
        action @ Action::SwitchToStack => (out.next_items, Some(action), preview_type),
        Action::TogglePreview => (out.next_items, None, preview_type.toggle()),
    };
    Ok(res)
}
