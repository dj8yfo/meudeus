use kdl::KdlNode;
use anyhow::anyhow;
use std::collections::{HashMap, HashSet};

use crate::{impl_try_from_kdl_node_uniqueness_check, impl_from_self_into_action_hashmap};

use super::single_key::SingleKey;


#[derive(Debug, Clone)]
pub struct StackKeymap {
    pub toggle_preview_type: SingleKey,
    pub pop_note_from_stack: SingleKey,
    pub move_note_to_top_of_stack: SingleKey,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    TogglePreviewType,
    PopNoteFromStack,
    MoveNoteToStackTop,
}


impl_try_from_kdl_node_uniqueness_check!(
    StackKeymap,
    "stack",
    toggle_preview_type,
    pop_note_from_stack,
    move_note_to_top_of_stack
);

impl_from_self_into_action_hashmap!(StackKeymap, Action,
    Action::TogglePreviewType => toggle_preview_type,
    Action::PopNoteFromStack => pop_note_from_stack,
    Action::MoveNoteToStackTop => move_note_to_top_of_stack
);
