use bevy::prelude::*;

#[derive(Component)]
pub struct IslandGenerator {
    pub origin: IVec3,
    pub dimensions: UVec3,
    pub min_regions: u32,
    pub min_sub_regions: u32,
}

#[derive(Component)]
pub struct IslandGenDebugViz;

