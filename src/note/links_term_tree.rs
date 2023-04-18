use std::{collections::HashSet, fmt::Display};

use async_recursion::async_recursion;
use colored::Colorize;
use termtree::Tree;

use sqlx::Result as SqlxResult;

use crate::{
    config::{color::ColorScheme, ExternalCommands, SurfParsing},
    database::{Database, SqliteAsyncHandle},
    highlight::MarkdownStatic,
    link::Link,
};

use super::Note;

#[derive(Clone)]
pub enum NoteLinkTerm {
    Note(Note),
    Link(Link),
    LinkHint(usize, ColorScheme),
    Cycle(String, ColorScheme),
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

            Self::LinkHint(num, color) => {
                let c = color.links.unlisted;
                write!(
                    f,
                    "{}",
                    format!("[ has {num} links ]").truecolor(c.r, c.g, c.b)
                )
            }
            Self::Cycle(cycle, color) => {
                let c = color.links.cycle;
                write!(f, "⟳ {}", cycle.truecolor(c.r, c.g, c.b).to_string())
            }
        }
    }
}

impl Note {
    #[async_recursion]
    pub async fn construct_link_term_tree(
        &self,
        level: usize,
        mut all_reachable: HashSet<Note>,
        external_commands: ExternalCommands,
        surf_parsing: SurfParsing,
        db: SqliteAsyncHandle,
        md_static: MarkdownStatic,
        color_scheme: ColorScheme,
        straight: bool,
    ) -> SqlxResult<(Tree<NoteLinkTerm>, HashSet<Note>)> {
        let mut tree = Tree::new(NoteLinkTerm::Note(self.clone()));
        all_reachable.insert(self.clone());

        let forward_links = db
            .lock()
            .await
            .find_links_from(&self.name(), md_static, color_scheme, straight)
            .await?;

        for next in forward_links.into_iter().rev() {
            if all_reachable.contains(&next) {
                tree.push(Tree::new(NoteLinkTerm::Cycle(next.name(), color_scheme)));
            } else {
                let (next_tree, roundtrip_reachable) = next
                    .construct_link_term_tree(
                        level + 1,
                        all_reachable,
                        external_commands.clone(),
                        surf_parsing.clone(),
                        db.clone(),
                        md_static,
                        color_scheme,
                        straight,
                    )
                    .await?;
                all_reachable = roundtrip_reachable;
                tree.push(next_tree);
            }
        }
        let links = Link::parse(self, &surf_parsing, color_scheme)?;

        if links.len() > 0 {
            if level > 1 {
                tree.push(NoteLinkTerm::LinkHint(links.len(), color_scheme));
            } else {
                for link in links {
                    tree.push(NoteLinkTerm::Link(link));
                }
            }
        }

        Ok((tree, all_reachable))
    }
}
