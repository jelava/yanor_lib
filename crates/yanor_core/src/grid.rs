use bevy::prelude::*;

use crate::index::{ComponentIndexPlugin, SparseComponentIndex};

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[component(immutable, storage = "SparseSet")]
pub struct GridPosition(pub IVec3);

impl GridPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}

// For now not going to try to deal with multi-cell shaped entities
// #[derive(Component, Default)]
// #[component(immutable)]
// pub enum GridShape {
//     #[default]
//     SingleCell,
// }

/// Useful for keeping track of locations of things that are scattered across a wide area with no extra
/// memory overhead, or which don't need to be efficiently accessible in sequence (an underlying data
/// structure with better spatial locality will do better for that).
pub type SparseGridIndex = SparseComponentIndex<GridPosition>;

pub type SparseGridIndexPlugin = ComponentIndexPlugin<SparseGridIndex>;
