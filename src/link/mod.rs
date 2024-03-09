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
    Url(String),
    File {
        file: PathBuf,
    },
    Dir {
        dir: PathBuf,
    },
    Broken(PathBuf, Option<u64>),
    CodeBlock {
        code_block: String,
        syntax_label: String,
    },
    FileLine {
        file: PathBuf,
        line_number: u64,
    },
}

impl Open for Link {
    fn open(&self, mut cfg: OpenCfg) -> io::Result<Option<std::process::ExitStatus>> {
        match &self.link {
            Destination::Url(url) => {
                cfg.url_cmd.replace_matching_element("$URL", url);
                Ok(Some(
                    cmd(cfg.url_cmd.command, cfg.url_cmd.args).run()?.status,
                ))
            }

            Destination::File { file, .. } => {
                let file_cmd = PathBuf::from(&cfg.file_cmd.command);
                let file_cmd = env_substitute::substitute(file_cmd);
                cfg.file_cmd
                    .replace_matching_element("$FILE", file.to_str().unwrap_or("bad utf path"));
                Ok(Some(
                    cmd(file_cmd.to_str().unwrap().to_owned(), cfg.file_cmd.args)
                        .run()?
                        .status,
                ))
            }

            Destination::FileLine { file, line_number } => {
                let file_cmd = PathBuf::from(&cfg.file_jump_cmd.command);
                let file_cmd = env_substitute::substitute(file_cmd);
                let prev_dir = std::env::current_dir()?;

                let next_dir = file.parent();
                if let Some(next_dir) = next_dir {
                    std::env::set_current_dir(next_dir)?;
                }

                cfg.file_jump_cmd
                    .replace_in_matching_element("$FILE", file.to_str().unwrap_or("bad utf path"));

                cfg.file_jump_cmd
                    .replace_in_matching_element("$LINE", &format!("{}", line_number));

                cfg.file_jump_cmd
                    .replace_in_matching_element("$COLUMN", &format!("{}", 1));
                let status = cmd(
                    file_cmd.to_str().unwrap().to_owned(),
                    cfg.file_jump_cmd.args,
                )
                .run()?
                .status;
                std::env::set_current_dir(prev_dir)?;

                Ok(Some(status))
            }
            Destination::Dir { dir, .. } => {
                cfg.dir_cmd
                    .replace_matching_element("$DIR", dir.to_str().unwrap_or("bad utf path"));
                Ok(Some(
                    cmd(cfg.dir_cmd.command, cfg.dir_cmd.args).run()?.status,
                ))
            }
            Destination::Broken(broken, _line) => {
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

    fn open_xdg(&self) -> Result<(), opener::OpenError> {
        match &self.link {
            Destination::Url(url) => opener::open(url),

            Destination::File { file, .. } | Destination::FileLine { file, .. } => {
                opener::open(file)
            }

            Destination::Dir { dir, .. } => opener::open(dir),
            Destination::Broken(broken, _line) => {
                eprintln!(
                    "{}",
                    format_two_tokens(
                        "cannot open broken: ",
                        broken.to_str().unwrap_or("bad utf8")
                    )
                );
                Err(opener::OpenError::Io(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not possible for broken path",
                )))
            }
            Destination::CodeBlock { .. } => {
                eprintln!(
                    "{}",
                    format_two_tokens("cannot open code_block: ", "code_block")
                );
                Ok(())
            }
        }
    }
}

impl Jump for Link {
    fn jump(
        &self,
        mut cfg: crate::config::Open,
    ) -> std::io::Result<Option<std::process::ExitStatus>> {
        let position = self.start;

        let file_cmd = PathBuf::from(&cfg.file_jump_cmd.command);
        let file_cmd = env_substitute::substitute(file_cmd);

        cfg.file_jump_cmd.replace_in_matching_element(
            "$FILE",
            self.containing_file_name.to_str().unwrap_or("bad utf path"),
        );

        cfg.file_jump_cmd
            .replace_in_matching_element("$LINE", &format!("{}", position.line));

        cfg.file_jump_cmd
            .replace_in_matching_element("$COLUMN", &format!("{}", position.column));

        Ok(Some(
            cmd(
                file_cmd.to_str().unwrap().to_owned(),
                cfg.file_jump_cmd.args,
            )
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
            .truecolor(parent_rgb.0.r, parent_rgb.0.g, parent_rgb.0.b)
            .to_string();
        let description = match self.link {
            Destination::Url(..) => {
                let url_rgb = self.color_scheme.links.url;
                self.description
                    .truecolor(url_rgb.0.r, url_rgb.0.g, url_rgb.0.b)
                    .to_string()
            }
            Destination::File { .. } | Destination::FileLine { .. } => {
                let file_rgb = self.color_scheme.links.file;
                self.description
                    .truecolor(file_rgb.0.r, file_rgb.0.g, file_rgb.0.b)
                    .to_string()
            }
            Destination::Dir { .. } => {
                let dir_rgb = self.color_scheme.links.dir;
                self.description
                    .truecolor(dir_rgb.0.r, dir_rgb.0.g, dir_rgb.0.b)
                    .to_string()
            }
            Destination::Broken(..) => {
                let broken_rgb = self.color_scheme.links.broken;
                self.description
                    .truecolor(broken_rgb.0.r, broken_rgb.0.g, broken_rgb.0.b)
                    .to_string()
            }
            Destination::CodeBlock { .. } => {
                let code_block_rgb = self.color_scheme.links.code_block;
                self.description
                    .truecolor(code_block_rgb.0.r, code_block_rgb.0.g, code_block_rgb.0.b)
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

    fn fs_link(
        description: String,
        link: String,
        parent_note: PathBuf,
        parent_name: String,
        has_line_suffix: &Regex,
        start: EditorPosition,
        color_scheme: ColorScheme,
    ) -> Self {
        let (link, line_suffix) = if has_line_suffix.is_match(&link) {
            let (link, suffix) = link.rsplit_once(':').unwrap();
            let suffix = suffix.parse::<u64>().ok();
            (link.to_string(), suffix)
        } else {
            (link, None)
        };
        let link = PathBuf::from(&link);
        let mut link = env_substitute::substitute(link);
        if link.is_relative() {
            link = parent_note.as_path().parent().unwrap().join(link);
        }
        match (link.is_file(), link.is_dir(), line_suffix) {
            (true, false, None) => Self {
                parent_name,
                description,
                link: Destination::File { file: link },
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            },
            (true, false, Some(line_suffix)) => Self {
                parent_name,
                description,
                link: Destination::FileLine {
                    file: link,
                    line_number: line_suffix,
                },
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            },
            (false, true, None) => Self {
                parent_name,
                description,
                link: Destination::Dir { dir: link },
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            },
            (false, true, Some(line)) => Self {
                parent_name,
                description,
                link: Destination::Broken(link, Some(line)),
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            },
            _ => Self {
                parent_name,
                description,
                link: Destination::Broken(link, None),
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            },
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        description: String,
        link: String,
        parent_note: PathBuf,
        parent_name: String,
        url: &Regex,
        has_line_suffix: &Regex,
        start: EditorPosition,
        color_scheme: ColorScheme,
    ) -> Self {
        if url.is_match(&link) {
            Self {
                parent_name,
                description,
                link: Destination::Url(link),
                preview_item: None,
                display_item: None,
                start,
                containing_file_name: parent_note,
                color_scheme,
            }
        } else {
            Self::fs_link(
                description,
                link,
                parent_note,
                parent_name,
                has_line_suffix,
                start,
                color_scheme,
            )
        }
    }
}
