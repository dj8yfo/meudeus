use std::{collections::HashSet, fmt::Display};

use crate::{
    config::SurfParsing,
    database::{Database, SqliteAsyncHandle},
    task_item::TaskItem,
};
use async_recursion::async_recursion;
use colored::Colorize;
use termtree::Tree;

use super::Note;
use sqlx::Result as SqlxResult;

#[derive(Clone)]
pub enum NoteTaskItemTerm {
    Note(Note),
    Task(TaskItem),
    Cycle(String),
}

impl Display for NoteTaskItemTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note(note) => {
                write!(f, "{}", note)
            }

            Self::Task(task) => {
                write!(f, "{}", task.skim_display(false))
            }
            Self::Cycle(cycle) => {
                write!(f, "⟳ {}", cycle.truecolor(150, 75, 0).to_string())
            }
        }
    }
}

impl NoteTaskItemTerm {
    pub fn parse(input: &[TaskItem], group_by_top_level: bool) -> Vec<Tree<Self>> {
        let mut result = vec![];
        let mut subrange_end = 0;
        let mut index = 0;
        while index < input.len() {
            if group_by_top_level {
                if index < subrange_end {
                    index = subrange_end;
                    if index >= input.len() {
                        break;
                    }
                }
            }
            let mut tree = Tree::new(NoteTaskItemTerm::Task(input[index].clone()));
            let current_offset = input[index].nested_level;
            let subrange_start = index + 1;
            subrange_end = index + 1;
            if subrange_start < input.len() {
                while subrange_end < input.len()
                    && input[subrange_end].nested_level > current_offset
                {
                    subrange_end += 1;
                }
                let subslice = &input[subrange_start..subrange_end];
                let children = NoteTaskItemTerm::parse(subslice, true);
                for child in children {
                    tree.push(child);
                }
            }
            result.push(tree);
            index += 1;
        }
        result
    }
}

impl Note {
    #[async_recursion]
    pub async fn construct_task_item_term_tree(
        &self,
        mut all_reachable: HashSet<Note>,
        surf_parsing: SurfParsing,
        db: SqliteAsyncHandle,
    ) -> SqlxResult<(Tree<NoteTaskItemTerm>, HashSet<Note>)> {
        let mut tree = Tree::new(NoteTaskItemTerm::Note(self.clone()));
        all_reachable.insert(self.clone());

        let tasks = TaskItem::parse(self, &surf_parsing)?;

        let trees = NoteTaskItemTerm::parse(&tasks, true);
        for task in trees {
            tree.push(task);
        }

        let forward_links = db.lock().await.find_links_from(&self.name()).await?;

        for next in forward_links {
            if all_reachable.contains(&next) {
                tree.push(Tree::new(NoteTaskItemTerm::Cycle(next.name())));
            } else {
                let (next_tree, roundtrip_reachable) = next
                    .construct_task_item_term_tree(all_reachable, surf_parsing.clone(), db.clone())
                    .await?;
                all_reachable = roundtrip_reachable;
                tree.push(next_tree);
            }
        }

        Ok((tree, all_reachable))
    }
}
