use crate::config::KdlNodeErrorType;
use kdl::KdlNode;
use std::collections::{HashMap, HashSet};

use crate::{impl_from_self_into_action_hashmap, impl_try_from_kdl_node_uniqueness_check};

use super::single_key::SingleKey;

#[derive(Debug, Clone)]
pub struct StackKeymap {
    pub toggle_preview_type: SingleKey,
    pub pop_note_from_stack: SingleKey,
    pub move_note_to_top_of_stack: SingleKey,
    pub return_to_explore: SingleKey,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    TogglePreviewType,
    PopNoteFromStack,
    MoveNoteToStackTop,
    ReturnToExplore,
}

impl_try_from_kdl_node_uniqueness_check!(
    StackKeymap,
    "world.keymap.stack",
    toggle_preview_type,
    pop_note_from_stack,
    move_note_to_top_of_stack,
    return_to_explore
);

impl_from_self_into_action_hashmap!(StackKeymap, Action,
    Action::TogglePreviewType => toggle_preview_type,
    Action::PopNoteFromStack => pop_note_from_stack,
    Action::MoveNoteToStackTop => move_note_to_top_of_stack,
    Action::ReturnToExplore => return_to_explore
);
