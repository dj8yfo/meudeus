use crate::impl_try_from_kdl_node_tagged;
use kdl::KdlNode;
use std::collections::HashMap;
use super::KdlNodeErrorType;

use super::ConfigPath;

use config_color::ConfigRGB;

mod config_color;

#[derive(Debug, Clone)]
pub struct Color {
    pub theme: ConfigPath,
    pub elements: ColorScheme,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorScheme {
    pub links: Links,
    pub notes: Notes,
}

#[derive(Debug, Clone, Copy)]
pub struct Notes {
    pub tag: ConfigRGB,
    pub special_tag: ConfigRGB,
}

#[derive(Debug, Clone, Copy)]
pub struct Links {
    pub parent_name: ConfigRGB,
    pub url: ConfigRGB,
    pub file: ConfigRGB,
    pub dir: ConfigRGB,
    pub broken: ConfigRGB,
    pub code_block: ConfigRGB,
    pub unlisted: ConfigRGB,
    pub cycle: ConfigRGB,
}

impl_try_from_kdl_node_tagged!(Color, "world.color",
    "theme" => theme,
    "elements" => elements
);

impl_try_from_kdl_node_tagged!(ColorScheme, "world.color.elements",
    "links" => links,
    "notes" => notes
);

impl_try_from_kdl_node_tagged!(Notes, "world.color.elements.notes",
    "tag" => tag,
    "special_tag" => special_tag
);

impl_try_from_kdl_node_tagged!(Links, "world.color.elements.links",
    "parent_name" => parent_name,
    "url" => url,
    "file" => file,
    "dir" => dir,
    "broken" => broken,
    "code_block" => code_block,
    "unlisted" => unlisted,
    "cycle" => cycle
);
