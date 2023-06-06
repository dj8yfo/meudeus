use super::KdlNodeErrorType;
use crate::impl_try_from_kdl_node_tagged;

use self::cmd_template::CmdTemplate;
pub mod cmd_template;
use kdl::KdlNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExternalCommands {
    pub preview: Preview,
    pub open: Open,
}

impl_try_from_kdl_node_tagged!(ExternalCommands, "world.external-commands", 
    "preview" => preview,
    "open" => open);

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

impl_try_from_kdl_node_tagged!(Preview, "world.external-commands.preview",
    "dir" => dir_cmd,
    "file" => file_cmd,
    "file-line" => file_line_cmd
);

impl_try_from_kdl_node_tagged!(Open, "world.external-commands.open",
    "file" => file_cmd,
    "file-jump" => file_jump_cmd,
    "dir" =>  dir_cmd,
    "url" => url_cmd,
    "pipe-$SNIPPET_TEXT-into" => pipe_text_snippet_cmd
);
