use std::{collections::HashSet, fmt::Display, fs};

use crate::{
    config::{color::ColorScheme, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    lines::find_position,
    task_item::TaskItem,
    Jump,
};
use async_recursion::async_recursion;
use colored::Colorize;
use syntect::easy::HighlightLines;
use termtree::Tree;

use super::Note;
use duct::cmd;
use sqlx::Result as SqlxResult;

#[derive(Clone)]
pub enum NoteTaskItemTerm {
    Note(Note),
    Task(TaskItem),
    TaskMono(TaskItem),
    TaskHint(usize, ColorScheme),
    Cycle(String, ColorScheme),
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
            Self::TaskHint(num, color) => {
                let c = color.links.unlisted;

                write!(
                    f,
                    "{}",
                    format!("[ has {num} task items ]").truecolor(c.r, c.g, c.b)
                )
            }
            Self::TaskMono(task) => {
                write!(f, "{}", task.skim_display_mono(false))
            }
            Self::Cycle(cycle, color) => {
                let c = color.links.cycle;
                write!(f, "⟳ {}", cycle.truecolor(c.r, c.g, c.b).to_string())
            }
        }
    }
}

impl NoteTaskItemTerm {
    pub fn parse(input: &[TaskItem], group_by_top_level: bool, mono: bool) -> Vec<Tree<Self>> {
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
            let mut tree;
            if mono {
                tree = Tree::new(NoteTaskItemTerm::TaskMono(input[index].clone()));
            } else {
                tree = Tree::new(NoteTaskItemTerm::Task(input[index].clone()));
            }
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
                let height_task = subrange_end - index;
                match tree.root {
                    NoteTaskItemTerm::Note(..) => unreachable!("note"),
                    NoteTaskItemTerm::Cycle(..) => unreachable!("cycle"),
                    NoteTaskItemTerm::TaskHint(_num, ..) => unreachable!("hint"),
                    NoteTaskItemTerm::Task(ref mut task)
                    | NoteTaskItemTerm::TaskMono(ref mut task) => {
                        task.next_index = Some(task.self_index + height_task);
                    }
                }
                let children = NoteTaskItemTerm::parse(subslice, true, mono);
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

impl Jump for NoteTaskItemTerm {
    fn jump(
        &self,
        mut cfg: crate::config::Open,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let task = match self {
            NoteTaskItemTerm::Note(..) => unreachable!("not expecting a note here"),
            NoteTaskItemTerm::Cycle(..) => unreachable!("not expecting a cycle here"),
            NoteTaskItemTerm::TaskHint(_num, ..) => unreachable!("hint"),
            NoteTaskItemTerm::Task(task) => task.clone(),
            NoteTaskItemTerm::TaskMono(task) => task.clone(),
        };

        let initial_contents: &str = &fs::read_to_string(&task.file_name)?;
        let offset = task.checkmark_offsets_in_string.start;
        let position = find_position(initial_contents, offset);

        cfg.file_jump_cmd.replace_in_matching_element(
            "$FILE",
            task.file_name.to_str().unwrap_or("bad utf path"),
        );

        cfg.file_jump_cmd
            .replace_in_matching_element("$LINE", &format!("{}", position.line));

        cfg.file_jump_cmd
            .replace_in_matching_element("$COLUMN", &format!("{}", position.column));

        Ok(Some(
            cmd(cfg.file_jump_cmd.command, cfg.file_jump_cmd.args)
                .run()?
                .status,
        ))
    }
}

impl Note {
    #[async_recursion]
    pub async fn construct_task_item_term_tree(
        &self,
        level: usize,
        mut all_reachable: HashSet<Note>,
        surf_parsing: SurfParsing,
        db: SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
    ) -> SqlxResult<(Tree<NoteTaskItemTerm>, HashSet<Note>)> {
        let mut tree = Tree::new(NoteTaskItemTerm::Note(self.clone()));
        all_reachable.insert(self.clone());

        let tasks = {
            let mut highlighter = HighlightLines::new(md_static.1, md_static.2);
            TaskItem::parse(self, &surf_parsing, &mut highlighter, md_static)?
        };

        let trees = NoteTaskItemTerm::parse(&tasks, true, false);
        if trees.len() > 0 {
            if level > 1 {
                tree.push(NoteTaskItemTerm::TaskHint(trees.len(), color_scheme));
            } else {
                for task in trees {
                    tree.push(task);
                }
            }
        }

        let forward_links = db
            .lock()
            .await
            .find_links_from(&self.name(), md_static, color_scheme)
            .await?;

        for next in forward_links.into_iter().rev() {
            if all_reachable.contains(&next) {
                tree.push(Tree::new(NoteTaskItemTerm::Cycle(
                    next.name(),
                    color_scheme,
                )));
            } else {
                let (next_tree, roundtrip_reachable) = next
                    .construct_task_item_term_tree(
                        level + 1,
                        all_reachable,
                        surf_parsing.clone(),
                        db.clone(),
                        md_static,
                        color_scheme,
                    )
                    .await?;
                all_reachable = roundtrip_reachable;
                tree.push(next_tree);
            }
        }

        Ok((tree, all_reachable))
    }
}
