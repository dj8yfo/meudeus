use std::collections::HashSet;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    note::PreviewType,
    print::format_two_tokens,
};

use colored::Colorize;

pub(crate) async fn exec(
    db: SqliteAsyncHandle,
    external_commands: ExternalCommands,
    surf_parsing: SurfParsing,
    name: Option<String>,
    md_static: MarkdownStatic,
    color_scheme: ColorScheme,
) -> Result<String, anyhow::Error> {
    let nested_threshold = 1;
    let note = {
        if let Some(name) = name {
            let note = db.lock().await.get(&name, md_static, color_scheme).await?;
            note
        } else {
            let list = db.lock().await.list(md_static, color_scheme).await?;
            let multi = false;
            crate::skim::open::Iteration::new(
                "print".to_string(),
                list,
                db.clone(),
                multi,
                PreviewType::Details,
                external_commands.clone(),
                surf_parsing.clone(),
                md_static,
                color_scheme,
                true,
                nested_threshold,
            )
            .run()
            .await?
        }
    };

    let straight = true;

    let (tree, _) = note
        .construct_link_term_tree(
            0,
            nested_threshold,
            HashSet::new(),
            external_commands,
            surf_parsing,
            db,
            md_static,
            color_scheme,
            straight,
        )
        .await?;

    println!("{}", tree);

    eprintln!(
        "{}",
        format_two_tokens("printed", &format!("{}", note.name()))
    );
    Ok("success".truecolor(0, 255, 255).to_string())
}
