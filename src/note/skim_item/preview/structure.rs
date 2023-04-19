use colored::Colorize;

use crate::{
    config::color::ColorScheme, database::SqliteAsyncHandle, highlight::MarkdownStatic, note::Note,
};

use std::collections::HashSet;

impl Note {
    pub async fn link_structure(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> String {
        let rs = self.resources().unwrap();

        let nested_threshold = 2;
        let result = self
            .construct_link_term_tree(
                0,
                nested_threshold,
                HashSet::new(),
                rs.external_commands.clone(),
                rs.surf_parsing.clone(),
                db.clone(),
                md_static,
                color_scheme,
                straight,
            )
            .await;

        match result {
            Ok((tree, _)) => format!("{}", tree),
            Err(err) => format!("db err {:?}", err).truecolor(255, 0, 0).to_string(),
        }
    }

    pub async fn task_structure(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> String {
        let rs = self.resources().unwrap();
        let nested_threshold = 2;
        let result = self
            .construct_task_item_term_tree(
                0,
                nested_threshold,
                HashSet::new(),
                rs.surf_parsing.clone(),
                db.clone(),
                md_static,
                color_scheme,
                straight,
            )
            .await;

        match result {
            Ok((tree, _)) => format!("{}", tree),
            Err(err) => format!("db err {:?}", err).truecolor(255, 0, 0).to_string(),
        }
    }
}
