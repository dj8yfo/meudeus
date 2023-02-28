use std::{fmt::Display, io, path::PathBuf};

use duct::cmd;
use regex::Regex;

use crate::{
    config::{cmd_template::CmdTemplate, ExternalCommands, Open as OpenCfg},
    print::print_two_tokens,
    Open,
};
mod skim_item;

#[derive(Clone, Debug)]
pub enum Destination {
    URL(String),
    File {
        file: PathBuf,
        preview: CmdTemplate,
    },
    Dir {
        dir: PathBuf,
        preview: CmdTemplate,
    },
    Broken(PathBuf),
    CodeBlock {
        code_block: String,
        syntax_label: String,
        open: CmdTemplate,
    },
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::URL(url) => write!(f, "{}", url),
            Self::File { file, .. } => write!(f, "{}", file.display()),
            Self::Dir { dir, .. } => write!(f, "{}", dir.display()),
            Self::Broken(broken) => write!(f, "{}", broken.display()),
            Self::CodeBlock {
                code_block,
                syntax_label,
                ..
            } => write!(
                f,
                "\n{}",
                skim_item::highlight_code_block(code_block.clone(), syntax_label)
            ),
        }
    }
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
                println!(
                    "{}",
                    print_two_tokens(
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
#[derive(Clone, Debug)]
pub struct Link {
    pub parent_name: String,
    pub description: String,
    pub link: Destination,
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} -> [{}] {}",
            self.parent_name, self.description, self.link
        )
    }
}

impl Link {
    pub fn new_code_block(
        parent_name: String,
        description: String,
        code_block: String,
        syntax_label: String,
        external_commands: &ExternalCommands,
    ) -> Self {
        Self {
            parent_name,
            description,
            link: Destination::CodeBlock {
                code_block,
                syntax_label,
                open: external_commands.open.file_cmd.clone(),
            },
        }
    }
    pub fn new(
        description: String,
        link: String,
        parent_note: PathBuf,
        parent_name: String,
        url: &Regex,
        external_commands: &ExternalCommands,
    ) -> Self {
        if url.is_match(&link) {
            Self {
                parent_name,
                description,
                link: Destination::URL(link),
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
                    link: Destination::File {
                        file: link,
                        preview: external_commands.preview.file_cmd.clone(),
                    },
                }
            } else if link.is_dir() {
                Self {
                    parent_name,
                    description,
                    link: Destination::Dir {
                        dir: link,
                        preview: external_commands.preview.dir_cmd.clone(),
                    },
                }
            } else {
                Self {
                    parent_name,
                    description,
                    link: Destination::Broken(link),
                }
            }
        }
    }
}
