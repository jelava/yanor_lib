use std::ops::Add;

use bevy::prelude::*;

use crate::index::{ComponentIndexPlugin, SparseComponentIndex};

#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[component(immutable, storage = "SparseSet")]
pub struct GridPos(pub IVec3);

impl GridPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }
}

impl From<GridPos> for Vec3 {
    fn from(value: GridPos) -> Self {
        let GridPos(ivec) = value;
        Vec3::new(ivec.x as f32, ivec.y as f32, ivec.z as f32)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum AxisDir {
    #[default]
    Zero,
    Plus,
    Minus,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct GridDir {
    pub x: AxisDir,
    pub y: AxisDir,
    pub z: AxisDir,
}

impl GridDir {
    pub const X: Self = Self::new(AxisDir::Plus, AxisDir::Zero, AxisDir::Zero);
    pub const NEG_X: Self = Self::new(AxisDir::Minus, AxisDir::Zero, AxisDir::Zero);

    pub const Y: Self = Self::new(AxisDir::Zero, AxisDir::Plus, AxisDir::Zero);
    pub const NEG_Y: Self = Self::new(AxisDir::Zero, AxisDir::Minus, AxisDir::Zero);

    pub const Z: Self = Self::new(AxisDir::Zero, AxisDir::Zero, AxisDir::Plus);
    pub const NEG_Z: Self = Self::new(AxisDir::Zero, AxisDir::Zero, AxisDir::Minus);

    // Not really a valid direction, but useful for checking validity of other GridDirs
    pub const ZERO: Self = Self::new(AxisDir::Zero, AxisDir::Zero, AxisDir::Zero);

    pub const fn new(x: AxisDir, y: AxisDir, z: AxisDir) -> Self {
        Self { x, y, z }
    }
}

impl From<GridDir> for IVec3 {
    fn from(value: GridDir) -> Self {
        use AxisDir::*;

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

impl TryFrom<IVec3> for GridDir {
    // TODO: actual error type (need to figure out error stuff more generally)
    type Error = &'static str;

    fn try_from(value: IVec3) -> std::result::Result<Self, Self::Error> {
        use AxisDir::*;

        let x_dir = match value.x {
            -1 => Minus,
            0 => Zero,
            1 => Plus,
            _ => return Err("x value too big/small"),
        };

        let y_dir = match value.y {
            -1 => Minus,
            0 => Zero,
            1 => Plus,
            _ => return Err("y value too big/small"),
        };

        let z_dir = match value.z {
            -1 => Minus,
            0 => Zero,
            1 => Plus,
            _ => return Err("z value too big/small"),
        };

        Ok(GridDir::new(x_dir, y_dir, z_dir))
    }
}

impl Add<GridDir> for GridPos {
    type Output = Self;

    fn add(self, rhs: GridDir) -> Self::Output {
        Self(self.0 + IVec3::from(rhs))
    }
}

#[derive(Component)]
pub enum CardinalGridDir {
    X,
    NegX,
    Y,
    NegY,
    Z,
    NegZ,
}

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub enum XzPlaneOrientation {
    #[default]
    FacingZ,
    FacingNegZ,
    FacingX,
    FacingNegX,
}

impl From<XzPlaneOrientation> for Dir3 {
    fn from(orientation: XzPlaneOrientation) -> Self {
        use XzPlaneOrientation::*;

        match orientation {
            FacingZ => Dir3::Z,
            FacingNegZ => Dir3::NEG_Z,
            FacingX => Dir3::X,
            FacingNegX => Dir3::NEG_X,
        }
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
pub type SparseGridIndex = SparseComponentIndex<GridPos>;

pub type SparseGridIndexPlugin = ComponentIndexPlugin<SparseGridIndex>;
