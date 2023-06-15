use kdl::KdlNode;

use super::single_key::SingleKey;
use crate::config::KdlNodeErrorType;
use crate::{impl_from_self_into_action_hashmap, impl_try_from_kdl_node_uniqueness_check};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct SurfKeymap {
    pub open_xdg: SingleKey,
    pub jump_to_link_or_snippet: SingleKey,
    pub return_to_explore: SingleKey,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    OpenXDG,
    JumpToLinkOrSnippet,
    ReturnToExplore,
}

impl_try_from_kdl_node_uniqueness_check!(
    SurfKeymap,
    "world.keymap.surf",
    open_xdg,
    jump_to_link_or_snippet,
    return_to_explore
);

impl_from_self_into_action_hashmap!(SurfKeymap, Action,
    Action::OpenXDG => open_xdg | "accept".to_string(),
    Action::JumpToLinkOrSnippet => jump_to_link_or_snippet | "accept".to_string(),
    Action::ReturnToExplore => return_to_explore | "accept".to_string()
);
