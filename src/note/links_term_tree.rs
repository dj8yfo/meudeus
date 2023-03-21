use std::{collections::HashSet, fmt::Display};

use async_recursion::async_recursion;
use colored::Colorize;
use termtree::Tree;

use sqlx::Result as SqlxResult;

use crate::{
    config::{ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    link::Link,
};

use super::Note;

#[derive(Clone)]
pub enum NoteLinkTerm {
    Note(Note),
    Link(Link),
    Cycle(String),
}

impl Display for NoteLinkTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note(note) => {
                write!(f, "{}", note)
            }

            Self::Link(link) => {
                write!(f, "{}", link.skim_display())
            }
            Self::Cycle(cycle) => {
                write!(f, "⟳ {}", cycle.truecolor(150, 75, 0).to_string())
            }
        }
    }
}

impl Note {
    #[async_recursion]
    pub async fn construct_link_term_tree(
        &self,
        mut all_reachable: HashSet<Note>,
        external_commands: ExternalCommands,
        surf_parsing: SurfParsing,
        db: SqliteAsyncHandle,
    ) -> SqlxResult<(Tree<NoteLinkTerm>, HashSet<Note>)> {
        let mut tree = Tree::new(NoteLinkTerm::Note(self.clone()));
        all_reachable.insert(self.clone());

        let forward_links = db.lock().await.find_links_from(&self.name()).await?;

        for next in forward_links {
            if all_reachable.contains(&next) {
                tree.push(Tree::new(NoteLinkTerm::Cycle(next.name())));
            } else {
                let (next_tree, roundtrip_reachable) = next
                    .construct_link_term_tree(
                        all_reachable,
                        external_commands.clone(),
                        surf_parsing.clone(),
                        db.clone(),
                    )
                    .await?;
                all_reachable = roundtrip_reachable;
                tree.push(next_tree);
            }
        }
        let links = Link::parse(self, &surf_parsing, &external_commands)?;
        for link in links {
            tree.push(NoteLinkTerm::Link(link));
        }

        Ok((tree, all_reachable))
    }
}