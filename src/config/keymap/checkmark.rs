use crate::config::KdlNodeErrorType;
use kdl::KdlNode;

use crate::{impl_from_self_into_action_hashmap, impl_try_from_kdl_node_uniqueness_check};
use std::collections::{HashMap, HashSet};

use super::single_key::SingleKey;

#[derive(Debug, Clone)]
pub struct CheckmarkKeymap {
    pub jump_to_task: SingleKey,
    pub copy_task_subtree_into_clipboard: SingleKey,
    pub widen_context_to_all_tasks: SingleKey,
    pub narrow_context_to_selected_task_subtree: SingleKey,
    pub return_to_explore: SingleKey,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    JumpToTask,
    CopyTaskSubtree,
    WidenContext,
    NarrowContext,
    ReturnToExplore,
}

impl_try_from_kdl_node_uniqueness_check!(
    CheckmarkKeymap,
    "world.keymap.checkmark",
    jump_to_task,
    copy_task_subtree_into_clipboard,
    widen_context_to_all_tasks,
    narrow_context_to_selected_task_subtree,
    return_to_explore
);

impl_from_self_into_action_hashmap!(CheckmarkKeymap, Action,
    Action::JumpToTask => jump_to_task | "accept".to_string(),
    Action::CopyTaskSubtree => copy_task_subtree_into_clipboard | "accept".to_string(),
    Action::WidenContext => widen_context_to_all_tasks | "accept".to_string(),
    Action::NarrowContext => narrow_context_to_selected_task_subtree | "accept".to_string(),
    Action::ReturnToExplore => return_to_explore | "accept".to_string()
);
