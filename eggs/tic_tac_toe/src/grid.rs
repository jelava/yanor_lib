use bevy::prelude::*;

use crate::index::{ComponentIndexPlugin, SparseComponentIndex};

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[require(GridShape)]
pub struct GridPosition(pub IVec3);

#[derive(Component, Default)]
pub enum GridShape {
    #[default]
    SingleCell,
}

/// Useful for keeping track of locations of things that are scattered across a wide area with no extra
/// memory overhead, or which don't need to be efficiently accessible in sequence (an underlying data
/// structure with better spatial locality will do better for that).
pub type SparseGridIndex = SparseComponentIndex<GridPosition>;

pub type SparseGridIndexPlugin = ComponentIndexPlugin<SparseGridIndex>;
