use colored::Colorize;

use crate::{database::SqliteAsyncHandle, highlight::MarkdownStatic, note::Note};

use std::collections::HashSet;

impl Note {
    pub async fn link_structure(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
    ) -> String {
        let rs = self.resources().unwrap();

        let result = self
            .construct_link_term_tree(
                0,
                HashSet::new(),
                rs.external_commands.clone(),
                rs.surf_parsing.clone(),
                db.clone(),
                md_static,
            )
            .await;

        match result {
            Ok((tree, _)) => format!("{}", tree),
            Err(err) => format!("db err {:?}", err).red().to_string(),
        }
    }

    pub async fn task_structure(
        &self,
        db: &SqliteAsyncHandle,
        md_static: MarkdownStatic,
    ) -> String {
        let rs = self.resources().unwrap();
        let result = self
            .construct_task_item_term_tree(
                0,
                HashSet::new(),
                rs.surf_parsing.clone(),
                db.clone(),
                md_static,
            )
            .await;

        match result {
            Ok((tree, _)) => format!("{}", tree),
            Err(err) => format!("db err {:?}", err).red().to_string(),
        }
    }
}
