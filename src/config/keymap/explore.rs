use crate::config::KdlNodeErrorType;
use kdl::KdlNode;
use std::collections::{HashMap, HashSet};

use crate::{impl_from_self_into_action_hashmap, impl_try_from_kdl_node_uniqueness_check};

use super::single_key::SingleKey;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Action {
    OpenXDG,
    PopulateSearchWithBacklinks,
    PopulateSearchWithForwardlinks,
    TogglePreviewType,
    WidenToAllNotes,
    SurfNoteSubtree,
    CheckmarkNote,
    RenameNote,
    LinkFromSelectedNote,
    UnlinkFromSelectedNote,
    RemoveNote,
    CreateAutolinkedNote,
    ToggleLinksDirection,
    SpliceReachableChildrenOfNote,
    NarrowSelection,
    DecreaseUnlistedThreshold,
    IncreaseUnlistedThreshold,
    PushNoteToStack,
    SwitchModeToStack,
}

#[derive(Debug, Clone)]
pub struct ExploreKeymap {
    pub open_xdg: SingleKey,
    pub populate_search_with_backlinks: SingleKey,
    pub populate_search_with_forwardlinks: SingleKey,
    pub toggle_preview_type: SingleKey,
    pub widen_to_all_notes: SingleKey,
    pub surf_note_subtree: SingleKey,
    pub checkmark_note: SingleKey,
    pub rename_note: SingleKey,
    pub link_from_selected_note: SingleKey,
    pub unlink_from_selected_note: SingleKey,
    pub remove_note: SingleKey,
    pub create_autolinked_note: SingleKey,
    pub toggle_links_direction: SingleKey,
    pub splice_reachable_children_of_note: SingleKey,
    pub narrow_selection: SingleKey,
    pub decrease_unlisted_threshold: SingleKey,
    pub increase_unlisted_threshold: SingleKey,
    pub push_note_to_stack: SingleKey,
    pub switch_mode_to_stack: SingleKey,
}

impl_try_from_kdl_node_uniqueness_check!(
    ExploreKeymap,
    "world.keymap.explore",
    open_xdg,
    populate_search_with_backlinks,
    populate_search_with_forwardlinks,
    toggle_preview_type,
    widen_to_all_notes,
    surf_note_subtree,
    checkmark_note,
    rename_note,
    link_from_selected_note,
    unlink_from_selected_note,
    remove_note,
    create_autolinked_note,
    toggle_links_direction,
    splice_reachable_children_of_note,
    narrow_selection,
    decrease_unlisted_threshold,
    increase_unlisted_threshold,
    push_note_to_stack,
    switch_mode_to_stack
);

impl_from_self_into_action_hashmap!(ExploreKeymap, Action,
    Action::OpenXDG => open_xdg | "accept".to_string(),
    Action::PopulateSearchWithBacklinks => populate_search_with_backlinks | "accept".to_string(),
    Action::PopulateSearchWithForwardlinks => populate_search_with_forwardlinks | "accept".to_string(),
    Action::TogglePreviewType => toggle_preview_type | "accept".to_string(),
    Action::WidenToAllNotes => widen_to_all_notes | "accept".to_string(),
    Action::SurfNoteSubtree => surf_note_subtree | "accept".to_string(),
    Action::CheckmarkNote => checkmark_note | "accept".to_string(),
    Action::RenameNote => rename_note | "accept".to_string(),
    Action::LinkFromSelectedNote => link_from_selected_note | "accept".to_string(),
    Action::UnlinkFromSelectedNote => unlink_from_selected_note | "accept".to_string(),
    Action::RemoveNote => remove_note | "accept".to_string(),
    Action::CreateAutolinkedNote => create_autolinked_note | "accept".to_string(),
    Action::ToggleLinksDirection => toggle_links_direction | "accept".to_string(),
    Action::SpliceReachableChildrenOfNote => splice_reachable_children_of_note | "accept".to_string(),
    Action::NarrowSelection => narrow_selection | "accept".to_string(),
    Action::DecreaseUnlistedThreshold => decrease_unlisted_threshold | "accept".to_string(),
    Action::IncreaseUnlistedThreshold => increase_unlisted_threshold | "accept".to_string(),
    Action::PushNoteToStack => push_note_to_stack | "accept".to_string(),
    Action::SwitchModeToStack => switch_mode_to_stack | "accept".to_string()
);
