use bevy::prelude::*;
use yanor_core::grid::*;

#[derive(Component)]
#[require(GridPos, XzPlaneOrientation)]
pub struct Chest;
