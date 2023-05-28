use kdl::KdlNode;

use super::single_key::SingleKey;
use crate::{impl_from_self_into_action_hashmap, impl_try_from_kdl_node_uniqueness_check};
use anyhow::anyhow;
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
    "surf",
    open_xdg,
    jump_to_link_or_snippet,
    return_to_explore
);

impl_from_self_into_action_hashmap!(SurfKeymap, Action,
    Action::OpenXDG => open_xdg,
    Action::JumpToLinkOrSnippet => jump_to_link_or_snippet,
    Action::ReturnToExplore => return_to_explore
);
