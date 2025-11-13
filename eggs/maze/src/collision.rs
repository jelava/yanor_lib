use bevy::prelude::*;
// use yanor_core::{grid::{GridDir, GridPos, SparseGridIndex}, index::ComponentIndex};

#[derive(Clone, Copy, Default)]
pub enum ColliderShape {
    #[default]
    Block,
    Slope, // mainly for stairs (and actual slopes)
}

#[derive(Component, Default)]
pub struct Collider {
    pub shape: ColliderShape,
    pub traversable: bool,
}

// #[derive(Component, Default)]
// pub struct Traversable;

// test whether stepping out of a particular grid cell is possible
// fn test_step_from<const N: usize>(
//     from_pos: &GridPos,
//     step_dir: &GridDir,
//     grid_index: Res<SparseGridIndex>,
//     collider_query: Query<&Collider, With<GridPos>>
// ) -> bool {
//     if let Some(&from_entities) = grid_index.get(from_pos) {
//         // let array: [Entity; N] = from_entities
//         //     .iter()
//         //     .map(|&entity| entity)
//         //     .collect();

//         // if let Ok(colliders) = collider_query.get_many(from_entities.iter().cloned().collect()) {

//         // }

//         todo!()
//     } else {
//         true
//     }
// }
