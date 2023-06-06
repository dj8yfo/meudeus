use crate::{config::KdlNodeErrorType, impl_try_from_kdl_node};
use std::collections::HashMap;

use kdl::KdlNode;

use self::{
    checkmark::CheckmarkKeymap, explore::ExploreKeymap, stack::StackKeymap, surf::SurfKeymap,
};
pub mod single_key;

pub mod surf;
pub mod checkmark;
pub mod stack;
pub mod explore;

#[derive(Debug, Clone)]
pub struct Keymap {
    pub surf: SurfKeymap,
    pub checkmark: CheckmarkKeymap,
    pub stack: StackKeymap,
    pub explore: ExploreKeymap,
}

impl_try_from_kdl_node!(Keymap, "world.keymap", surf, checkmark, stack, explore);
