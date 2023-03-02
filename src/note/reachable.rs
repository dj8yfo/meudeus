use std::{collections::HashSet, fmt::Display};

use sqlx::Result as SqlxResult;

use crate::database::{Database, SqliteAsyncHandle};
use colored::Colorize;

use super::Note;
use termtree::Tree;

use async_recursion::async_recursion;

#[derive(Clone)]
pub enum NoteTerm {
    Note(Note),
    Cycle(String),
}

impl Display for NoteTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note(note) => {
                write!(f, "{}", note)
            }
            Self::Cycle(cycle) => {
                write!(f, "⟳ {}", cycle.truecolor(150, 75, 0).to_string())
            }
        }
    }
}
impl super::Note {
    pub async fn reachable_notes(&self, db: SqliteAsyncHandle) -> SqlxResult<Vec<Self>> {
        let mut reachable_all: HashSet<Note> = HashSet::new();
        let mut current_layer: HashSet<Note> = HashSet::new();
        current_layer.insert(self.clone());

        loop {
            let mut next_layer: HashSet<Note> = HashSet::new();

            let lock = db.lock().await;
            for note in &current_layer {
                let forward_links = lock.find_links_from(&note.name()).await?;
                next_layer.extend(forward_links);
            }
            reachable_all.extend(current_layer.drain());
            let diff: HashSet<_> = next_layer.difference(&reachable_all).collect();
            if diff.is_empty() {
                break;
            }

            current_layer = next_layer;
        }
        let all_vec: Vec<_> = reachable_all.into_iter().collect();
        Ok(all_vec)
    }

    #[async_recursion]
    pub async fn construct_term_tree(
        &self,
        mut all_reachable: HashSet<Note>,
        db: SqliteAsyncHandle,
    ) -> SqlxResult<(Tree<NoteTerm>, HashSet<Note>)> {
        let mut tree = Tree::new(NoteTerm::Note(self.clone()));
        all_reachable.insert(self.clone());

        let forward_links = db.lock().await.find_links_from(&self.name()).await?;

        for next in forward_links {
            if all_reachable.contains(&next) {
                tree.push(Tree::new(NoteTerm::Cycle(next.name())));
            } else {
                let (next_tree, roundtrip_reachable) =
                    next.construct_term_tree(all_reachable, db.clone()).await?;
                all_reachable = roundtrip_reachable;
                tree.push(next_tree);
            }
        }

        Ok((tree, all_reachable))
    }
}
