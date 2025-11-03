use std::ops::Add;

use bevy::prelude::*;

use crate::index::{ComponentIndexPlugin, SparseComponentIndex};

#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[component(immutable, storage = "SparseSet")]
pub struct GridPosition(pub IVec3);

impl GridPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn from_vec_floor(value: Vec3) -> Self {
        Self(IVec3::new(value.x as i32, value.y as i32, value.z as i32))
    }
}

impl From<GridPosition> for Vec3 {
    fn from(value: GridPosition) -> Self {
        let GridPosition(ivec) = value;
        Vec3::new(ivec.x as f32, ivec.y as f32, ivec.z as f32)
    }
}

#[derive(Clone, Copy, Default)]
pub enum AxisDirection {
    #[default]
    Zero,
    Plus,
    Minus,
}

#[derive(Clone, Copy, Default)]
pub struct GridDirection {
    x: AxisDirection,
    y: AxisDirection,
    z: AxisDirection,
}

impl GridDirection {
    pub const X: Self = Self::new(AxisDirection::Plus, AxisDirection::Zero, AxisDirection::Zero);
    pub const NEG_X: Self = Self::new(AxisDirection::Minus, AxisDirection::Zero, AxisDirection::Zero);

    pub const Y: Self = Self::new(AxisDirection::Zero, AxisDirection::Plus, AxisDirection::Zero);
    pub const NEG_Y: Self = Self::new(AxisDirection::Zero, AxisDirection::Minus, AxisDirection::Zero);

    pub const Z: Self = Self::new(AxisDirection::Zero, AxisDirection::Zero, AxisDirection::Plus);
    pub const NEG_Z: Self = Self::new(AxisDirection::Zero, AxisDirection::Zero, AxisDirection::Minus);

    pub const fn new(x: AxisDirection, y: AxisDirection, z: AxisDirection) -> Self {
        Self { x, y, z }
    }
}

impl From<GridDirection> for IVec3 {
    fn from(value: GridDirection) -> Self {
        use AxisDirection::*;

        let x = match value.x {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        let y = match value.y {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        let z = match value.z {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        IVec3 { x, y, z }
    }
}

impl Add<GridDirection> for GridPosition {
    type Output = Self;

    fn add(self, rhs: GridDirection) -> Self::Output {
        Self(self.0 + IVec3::from(rhs))
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
