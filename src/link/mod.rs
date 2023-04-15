use std::{fmt::Display, io, path::PathBuf};

use colored::Colorize;
use comrak::nodes::Sourcepos;
use duct::cmd;
use regex::Regex;
use skim::AnsiString;

use crate::{
    config::{color::ColorScheme, Open as OpenCfg},
    lines::EditorPosition,
    print::format_two_tokens,
    Jump, Open,
};
mod parse;
mod skim_item;

#[derive(Clone, Debug)]
pub enum Destination {
    URL(String),
    File {
        file: PathBuf,
    },
    Dir {
        dir: PathBuf,
    },
    Broken(PathBuf),
    CodeBlock {
        code_block: String,
        syntax_label: String,
    },
}

impl Open for Link {
    fn open(&self, mut cfg: OpenCfg) -> io::Result<Option<std::process::ExitStatus>> {
        match &self.link {
            Destination::URL(url) => {
                cfg.url_cmd.replace_matching_element("$URL", url);
                Ok(Some(
                    cmd(cfg.url_cmd.command, cfg.url_cmd.args).run()?.status,
                ))
            }

            Destination::File { file, .. } => {
                cfg.file_cmd
                    .replace_matching_element("$FILE", file.to_str().unwrap_or("bad utf path"));
                Ok(Some(
                    cmd(cfg.file_cmd.command, cfg.file_cmd.args).run()?.status,
                ))
            }
            Destination::Dir { dir, .. } => {
                cfg.dir_cmd
                    .replace_matching_element("$DIR", dir.to_str().unwrap_or("bad utf path"));
                Ok(Some(
                    cmd(cfg.dir_cmd.command, cfg.dir_cmd.args).run()?.status,
                ))
            }
            Destination::Broken(broken) => {
                eprintln!(
                    "{}",
                    format_two_tokens(
                        "cannot open broken: ",
                        broken.to_str().unwrap_or("bad utf8")
                    )
                );
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not possible for broken path",
                ))
            }
            Destination::CodeBlock { code_block, .. } => Ok(Some(
                cmd(
                    cfg.pipe_text_snippet_cmd.command,
                    cfg.pipe_text_snippet_cmd.args,
                )
                .stdin_bytes(code_block.clone())
                .run()?
                .status,
            )),
        }
    }
}

impl Jump for Link {
    fn jump(
        &self,
        mut cfg: crate::config::Open,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let position = self.start;

        cfg.file_jump_cmd.replace_in_matching_element(
            "$FILE",
            self.containing_file_name.to_str().unwrap_or("bad utf path"),
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
#[derive(Clone, Debug)]
pub struct Link {
    pub containing_file_name: PathBuf,
    pub parent_name: String,
    pub description: String,
    pub link: Destination,
    pub display_item: Option<AnsiString<'static>>,
    pub preview_item: Option<String>,

    pub start: EditorPosition,
    color_scheme: ColorScheme,
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} -> [{}]", self.parent_name, self.description)
    }
}

impl Link {
    pub fn skim_display(&self) -> String {
        let parent_rgb = self.color_scheme.links.parent_name;
        let parent_name = self
            .parent_name
            .truecolor(parent_rgb.r, parent_rgb.g, parent_rgb.b)
            .to_string();
        let description = match self.link {
            Destination::URL(..) => {
                let url_rgb = self.color_scheme.links.url;
                self.description
                    .truecolor(url_rgb.r, url_rgb.g, url_rgb.b)
                    .to_string()
            }
            Destination::File { .. } => {
                let file_rgb = self.color_scheme.links.file;
                self.description
                    .truecolor(file_rgb.r, file_rgb.g, file_rgb.b)
                    .to_string()
            }
            Destination::Dir { .. } => {
                let dir_rgb = self.color_scheme.links.dir;
                self.description
                    .truecolor(dir_rgb.r, dir_rgb.g, dir_rgb.b)
                    .to_string()
            }
            Destination::Broken(..) => {
                let broken_rgb = self.color_scheme.links.broken;
                self.description
                    .truecolor(broken_rgb.r, broken_rgb.g, broken_rgb.b)
                    .to_string()
            }
            Destination::CodeBlock { .. } => {
                let code_block_rgb = self.color_scheme.links.code_block;
                self.description
                    .truecolor(code_block_rgb.r, code_block_rgb.g, code_block_rgb.b)
                    .to_string()
            }
        };

        let input = format!("{} -> [{}]", parent_name, description);
        input
    }
    pub fn new_code_block(
        parent_note: PathBuf,
        parent_name: String,
        description: String,
        code_block: String,
        syntax_label: String,
        source_position: Sourcepos,
        color_scheme: ColorScheme,
    ) -> Self {
        Self {
            parent_name,
            description,
            link: Destination::CodeBlock {
                code_block,
                syntax_label,
            },
            preview_item: None,
            display_item: None,
            start: EditorPosition {
                line: source_position.start.line,
                column: source_position.start.column,
            },
            containing_file_name: parent_note,
            color_scheme,
        }
    }
    pub fn new(
        description: String,
        link: String,
        parent_note: PathBuf,
        parent_name: String,
        url: &Regex,
        start: EditorPosition,
        color_scheme: ColorScheme,
    ) -> Self {
        if url.is_match(&link) {
            Self {
                parent_name,
                description,
                link: Destination::URL(link),
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            }
        } else {
            let mut link = PathBuf::from(&link);
            if link.is_relative() {
                link = parent_note.as_path().parent().unwrap().join(link);
            }
            if link.is_file() {
                Self {
                    parent_name,
                    description,
                    link: Destination::File { file: link },
                    preview_item: None,
                    display_item: None,
                    start,
                    containing_file_name: parent_note,
                    color_scheme,
                }
            } else if link.is_dir() {
                Self {
                    parent_name,
                    description,
                    link: Destination::Dir { dir: link },
                    preview_item: None,
                    display_item: None,
                    start,
                    containing_file_name: parent_note,
                    color_scheme,
                }
            } else {
                Self {
                    parent_name,
                    description,
                    link: Destination::Broken(link),
                    preview_item: None,
                    display_item: None,
                    start,
                    containing_file_name: parent_note,
                    color_scheme,
                }
            }
        }
    }
}
