use kdl::KdlNode;
use std::collections::{HashMap, HashSet};
use anyhow::anyhow;

use crate::{impl_try_from_kdl_node_uniqueness_check, impl_from_self_into_action_hashmap};

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
    DccreaseUnlistedThreshold,
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
    pub dccrease_unlisted_threshold: SingleKey,
    pub increase_unlisted_threshold: SingleKey,
    pub push_note_to_stack: SingleKey,
    pub switch_mode_to_stack: SingleKey,
}

impl_try_from_kdl_node_uniqueness_check!(
    ExploreKeymap,
    "explore",
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
    dccrease_unlisted_threshold,
    increase_unlisted_threshold,
    push_note_to_stack,
    switch_mode_to_stack
);

impl_from_self_into_action_hashmap!(ExploreKeymap, Action,
    Action::OpenXDG => open_xdg,
    Action::PopulateSearchWithBacklinks => populate_search_with_backlinks,
    Action::PopulateSearchWithForwardlinks => populate_search_with_forwardlinks,    
    Action::TogglePreviewType => toggle_preview_type,
    Action::WidenToAllNotes => widen_to_all_notes,
    Action::SurfNoteSubtree => surf_note_subtree,
    Action::CheckmarkNote => checkmark_note,
    Action::RenameNote => rename_note,
    Action::LinkFromSelectedNote => link_from_selected_note,
    Action::UnlinkFromSelectedNote => unlink_from_selected_note,
    Action::RemoveNote => remove_note,
    Action::CreateAutolinkedNote => create_autolinked_note,
    Action::ToggleLinksDirection => toggle_links_direction,
    Action::SpliceReachableChildrenOfNote => splice_reachable_children_of_note,
    Action::NarrowSelection => narrow_selection,
    Action::DccreaseUnlistedThreshold => dccrease_unlisted_threshold,
    Action::IncreaseUnlistedThreshold => increase_unlisted_threshold,
    Action::PushNoteToStack => push_note_to_stack,
    Action::SwitchModeToStack => switch_mode_to_stack
);
