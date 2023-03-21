use std::{fs, path::PathBuf};

use crate::{
    config::{ExternalCommands, SurfParsing},
    note::Note,
};

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

impl Link {
    fn reference_link_parse(
        note: &Note,
        result: &mut Vec<Link>,
        surf: &SurfParsing,
        external_commands: &ExternalCommands,
        file_path: &PathBuf,
        file_content: &str,
    ) {
        for link in surf
            .markdown_reference_link_regex
            .captures_iter(file_content)
        {
            result.push(Link::new(
                link["description"].to_string(),
                link["url"].to_string(),
                file_path.clone(),
                note.name(),
                &surf.url_regex,
                external_commands,
            ));
        }
    }

    fn ast_parse_code_blocks(
        note: &Note,
        result: &mut Vec<Link>,
        external_commands: &ExternalCommands,
        file_content: &str,
    ) {
        let arena = Arena::new();

        let root = parse_document(&arena, file_content, &ComrakOptions::default());

        let mut counter = 0;
        iter_nodes(
            root,
            &mut counter,
            &mut |node, counter| match &mut node.data.borrow_mut().value {
                &mut NodeValue::CodeBlock(ref mut block) => {
                    let syntax_label =
                        String::from_utf8(block.info.clone()).unwrap_or("bad_utf".to_string());
                    let code_block =
                        String::from_utf8(block.literal.clone()).unwrap_or("bad_utf".to_string());
                    let description = if let Some(line) = code_block.lines().next() {
                        line.to_string()
                    } else {
                        format!("snippet[{}]", counter)
                    };
                    result.push(Link::new_code_block(
                        note.name(),
                        description,
                        code_block,
                        syntax_label,
                        external_commands,
                    ));
                    *counter += 1;
                }
                _ => (),
            },
        );
    }
    pub fn parse(
        note: &Note,
        surf: &SurfParsing,
        external_commands: &ExternalCommands,
    ) -> std::io::Result<Vec<Link>> {
        if let Some(file_path) = note.file_path() {
            let mut result = vec![];
            let file_content = fs::read_to_string(file_path)?;

            Self::reference_link_parse(
                note,
                &mut result,
                surf,
                external_commands,
                file_path,
                &file_content,
            );
            Self::ast_parse_code_blocks(note, &mut result, external_commands, &file_content);

            Ok(result.into_iter().rev().collect())
        } else {
            return Ok(vec![]);
        }
    }
}
