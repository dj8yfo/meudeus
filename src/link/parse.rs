use std::{fs, path::PathBuf};

use crate::config::color::ColorScheme;
use crate::lines::find_position;
use crate::{config::SurfParsing, note::Note};

use super::Link;

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, ComrakOptions};

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, counter: &mut usize, f: &mut F)
where
    F: FnMut(&'a AstNode<'a>, &mut usize),
{
    f(node, counter);
    for c in node.children() {
        iter_nodes(c, counter, f);
    }
}

impl
    From<(
        regex::Captures<'_>,
        PathBuf,
        String,
        &'_ regex::Regex,
        &str,
        ColorScheme,
        &'_ regex::Regex,
    )> for Link
{
    fn from(
        value: (
            regex::Captures<'_>,
            PathBuf,
            String,
            &'_ regex::Regex,
            &str,
            ColorScheme,
            &'_ regex::Regex,
        ),
    ) -> Self {
        let captures = value.0;
        let start = captures.name("description").unwrap();
        let start = start.start();
        let start = find_position(value.4, start);
        Link::new(
            captures["description"].to_string(),
            captures["url"].to_string(),
            value.1,
            value.2,
            value.3,
            value.6,
            start,
            value.5,
        )
    }
}

impl Link {
    fn reference_link_parse(
        note: &Note,
        result: &mut Vec<Link>,
        surf: &SurfParsing,
        file_path: &PathBuf,
        file_content: &str,
        color_scheme: ColorScheme,
    ) {
        for link in surf
            .markdown_reference_link_regex
            .captures_iter(file_content)
        {
            result.push(
                (
                    link,
                    file_path.clone(),
                    note.name(),
                    &surf.url_regex,
                    file_content,
                    color_scheme,
                    &surf.has_line_regex,
                )
                    .into(),
            );
        }
    }

    fn ast_parse_code_blocks(
        note: &Note,
        result: &mut Vec<Link>,
        file_path: &PathBuf,
        file_content: &str,
        color_scheme: ColorScheme,
    ) {
        let arena = Arena::new();

        let root = parse_document(&arena, file_content, &ComrakOptions::default());

        let mut counter = 0;
        iter_nodes(root, &mut counter, &mut |node, counter| {
            let source_position = node.data.borrow().sourcepos;
            match &node.data.borrow().value {
                NodeValue::CodeBlock(ref block) => {
                    let syntax_label = block.info.clone();
                    let code_block = block.literal.clone();
                    let description = if let Some(line) = code_block.lines().next() {
                        line.to_string()
                    } else {
                        format!("snippet[{}]", counter)
                    };
                    result.push(Link::new_code_block(
                        file_path.clone(),
                        note.name(),
                        description,
                        code_block,
                        syntax_label,
                        source_position,
                        color_scheme,
                    ));
                    *counter += 1;
                }
                _ => (),
            }
        });
    }
    pub fn parse(
        note: &Note,
        surf: &SurfParsing,
        color_scheme: ColorScheme,
    ) -> std::io::Result<Vec<Link>> {
        if let Some(file_path) = note.file_path() {
            let mut result = vec![];
            let file_content = fs::read_to_string(file_path)?;

            Self::reference_link_parse(
                note,
                &mut result,
                surf,
                file_path,
                &file_content,
                color_scheme,
            );
            Self::ast_parse_code_blocks(note, &mut result, file_path, &file_content, color_scheme);

            Ok(result)
        } else {
            return Ok(vec![]);
        }
    }
}
