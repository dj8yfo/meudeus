use std::{fs, path::PathBuf};

use kdl::{KdlDocument, KdlNode};
use regex::Regex;

use crate::print::format_two_tokens;
use anyhow::anyhow;

use self::{cmd_template::CmdTemplate, color::ColorScheme};

pub mod cmd_template;
pub mod color;

static PROGRAM_NAME: &str = "mds";
#[derive(Debug)]
pub struct Config {
    pub work_dir: PathBuf,
    pub surf_parsing: SurfParsing,
    pub external_commands: ExternalCommands,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub struct SurfParsing {
    pub url_regex: Regex,
    pub markdown_reference_link_regex: Regex,
    pub task_item_regex: Regex,
    pub has_line_regex: Regex,
}

#[derive(Debug, Clone)]
pub struct ExternalCommands {
    pub preview: Preview,
    pub open: Open,
}
#[derive(Debug, Clone)]
pub struct Preview {
    pub dir_cmd: CmdTemplate,
    pub file_cmd: CmdTemplate,
    pub file_line_cmd: CmdTemplate,
}

#[derive(Debug, Clone)]
pub struct Open {
    pub file_cmd: CmdTemplate,
    pub file_jump_cmd: CmdTemplate,
    pub url_cmd: CmdTemplate,
    pub dir_cmd: CmdTemplate,
    pub pipe_text_snippet_cmd: CmdTemplate,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub theme: PathBuf,
    pub elements: ColorScheme,
}

impl TryFrom<&KdlNode> for Open {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let url = value
            .children()
            .ok_or(anyhow!("`open` should have children"))?
            .get_args("url");

        let url = url.try_into()?;

        let file = value
            .children()
            .ok_or(anyhow!("`open` should have children"))?
            .get_args("file");

        let file = file.try_into()?;

        let dir = value
            .children()
            .ok_or(anyhow!("`open` should have children"))?
            .get_args("dir");

        let dir = dir.try_into()?;
        let snippet = value
            .children()
            .ok_or(anyhow!("`open` should have children"))?
            .get_args("pipe-$SNIPPET_TEXT-into");

        let snippet = snippet.try_into()?;

        let file_jump = value
            .children()
            .ok_or(anyhow!("`open` should have children"))?
            .get_args("file-jump");

        let file_jump = file_jump.try_into()?;

        Ok(Self {
            url_cmd: url,
            file_cmd: file,
            file_jump_cmd: file_jump,
            dir_cmd: dir,
            pipe_text_snippet_cmd: snippet,
        })
    }
}
impl TryFrom<&KdlNode> for Preview {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let dir = value
            .children()
            .ok_or(anyhow!("`preview` should have children"))?
            .get_args("dir");

        let dir = dir.try_into()?;

        let file = value
            .children()
            .ok_or(anyhow!("`preview` should have children"))?
            .get_args("file");

        let file = file.try_into()?;

        let file_line = value
            .children()
            .ok_or(anyhow!("`preview` should have children"))?
            .get_args("file-line");

        let file_line = file_line.try_into().map_err(|err| {
            anyhow!(
                "`world.external-commands.preview.file-line` config field problem: {}",
                err
            )
        })?;

        Ok(Self {
            dir_cmd: dir,
            file_cmd: file,
            file_line_cmd: file_line,
        })
    }
}
impl TryFrom<&KdlNode> for ExternalCommands {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let preview = value
            .children()
            .ok_or(anyhow!("`external-commands` should have children"))?
            .get("preview")
            .ok_or(anyhow!("no `preview` in config"))?;

        let preview = preview.try_into()?;

        let open = value
            .children()
            .ok_or(anyhow!("`external-commands` should have children"))?
            .get("open")
            .ok_or(anyhow!("no `open` in config"))?;

        let open = open.try_into()?;

        Ok(Self { preview, open })
    }
}
impl TryFrom<&KdlNode> for SurfParsing {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let url_regex = value
            .children()
            .ok_or(anyhow!("`surf-parsing` should have children"))?
            .get("url-regex")
            .ok_or(anyhow!("no `url-regex` in config"))?
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let markdown_reference_link_regex = value
            .children()
            .ok_or(anyhow!("`surf-parsing` should have children"))?
            .get("markdown-reference-link-regex")
            .ok_or(anyhow!("no `markdown-reference-link-regex` in config"))?
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let task_item_regex = value
            .children()
            .ok_or(anyhow!("`surf-parsing` should have children"))?
            .get("task-item-regex")
            .ok_or(anyhow!("no `task-item-regex` in config"))?
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let file_dest_has_line_regex = value
            .children()
            .ok_or(anyhow!("`surf-parsing` should have children"))?
            .get("file-dest-has-line-regex")
            .ok_or(anyhow!("no `file-dest-has-line-regex` in config"))?
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let url_regex = Regex::new(&url_regex)?;
        let markdown_reference_link_regex = Regex::new(&markdown_reference_link_regex)?;
        let task_item_regex = Regex::new(&task_item_regex)?;
        let file_dest_has_line_regex = Regex::new(&file_dest_has_line_regex)?;

        Ok(Self {
            url_regex,
            markdown_reference_link_regex,
            task_item_regex,
            has_line_regex: file_dest_has_line_regex,
        })
    }
}

impl TryFrom<&KdlNode> for Color {
    type Error = anyhow::Error;

    fn try_from(value: &KdlNode) -> Result<Self, Self::Error> {
        let theme = value
            .children()
            .ok_or(anyhow!("`color` should have children"))?
            .get("theme")
            .ok_or(anyhow!("no `color.theme` node in config"))?;

        let string = theme
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        let elements = value
            .children()
            .ok_or(anyhow!("`color` should have children"))?
            .get("elements")
            .ok_or(anyhow!("no `elements` in config"))?;

        let elements = elements.try_into()?;

        Ok(Self {
            theme: PathBuf::from(string),
            elements,
        })
    }
}
impl Config {
    pub fn get_work_dir(doc: &KdlDocument) -> anyhow::Result<PathBuf> {
        let work_dir = doc
            .get("world")
            .ok_or(anyhow!("no `world` node in config"))?
            .children()
            .ok_or(anyhow!("`world` should have children"))?
            .get("notes-work-dir")
            .ok_or(anyhow!("no `world.notes-work-dir` node in config"))?;

        let string = work_dir
            .get(0)
            .ok_or(anyhow!("arg not found"))?
            .value()
            .as_string()
            .ok_or(anyhow!("should be string"))?
            .to_string();

        Ok(PathBuf::from(string))
    }

    pub fn parse() -> anyhow::Result<Self> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(PROGRAM_NAME)?;

        let config_path = xdg_dirs.get_config_file("config.kdl");
        eprintln!(
            "{} \n",
            format_two_tokens(
                "expected config path: ",
                config_path.to_str().unwrap_or("bad utf8")
            )
        );

        let config_file = fs::read_to_string(config_path)?;

        let doc: KdlDocument = config_file.parse()?;
        let work_dir = Self::get_work_dir(&doc)?;
        let surf_parsing = doc
            .get("world")
            .ok_or(anyhow!("no `world` node in config"))?
            .children()
            .ok_or(anyhow!("`world` should have children"))?
            .get("surf-parsing")
            .ok_or(anyhow!("no `world.surf-parsing` node in config"))?;

        let external_commands = doc
            .get("world")
            .ok_or(anyhow!("no `world` node in config"))?
            .children()
            .ok_or(anyhow!("`world` should have children"))?
            .get("external-commands")
            .ok_or(anyhow!("no `world.external-commands` node in config"))?;

        let color = doc
            .get("world")
            .ok_or(anyhow!("no `world` node in config"))?
            .children()
            .ok_or(anyhow!("`world` should have children"))?
            .get("color")
            .ok_or(anyhow!("no `world.color` node in config"))?;

        let surf_parsing = surf_parsing.try_into()?;
        let external_commands = external_commands.try_into()?;

        let color = color.try_into()?;
        Ok(Self {
            surf_parsing,
            work_dir,
            external_commands,
            color,
        })
    }
}
