mod common;

use common::*;

use bevy::prelude::*;
use yanor_core::{
    grid::{GridPosition, SparseGridIndex, SparseGridIndexPlugin},
    index::ComponentIndex,
};

// Generic set of basic tests for anything that implements GridIndex (specific tests below)

#[test]
fn basic_sparse_grid_index_tests() {
    App::new()
        .add_plugins(BaseTestPlugins)
        .add_plugins(SparseGridIndexPlugin::default())
        .add_systems(Startup, init_spawns)
        .add_systems(PostStartup, check_init)
        .add_systems(Update, tick)
        .add_systems(PostUpdate, (check_tick, end_test).chain())
        .run();
}

fn init_spawns(mut commands: Commands) {
    commands.spawn(GridPosition(IVec3::new(1, 2, 3)));
    commands.spawn(GridPosition(IVec3::new(0, 0, 0)));
}

fn check_init(grid_index: Res<SparseGridIndex>, pos_query: Query<&GridPosition>) {
    for pos in &pos_query {
        let pos_entities = grid_index.get(pos).expect("Expected to find entities");
        assert_eq!(pos_entities.len(), 1);
        assert!(pos_query.get(*pos_entities.iter().next().unwrap()).is_ok())
    }
}

fn tick(mut commands: Commands, grid_index: Res<SparseGridIndex>) {
    // move the position of the entity at (1, 2, 3) to test grid index update
    let entities_at_123 = grid_index
        .get(&GridPosition(IVec3::new(1, 2, 3)))
        .expect("Expected to find entity at (1, 2, 3)");

    assert_eq!(entities_at_123.len(), 1);

    let entity_at_123 = entities_at_123.iter().next().unwrap();

    commands
        .entity(*entity_at_123)
        .insert(GridPosition(IVec3::new(2, 3, 4)));

    let entities_at_000 = grid_index
        .get(&GridPosition(IVec3::new(0, 0, 0)))
        .expect("Expected to find entity at (0, 0, 0)");

    assert_eq!(entities_at_000.len(), 1);

    let entity_at_000 = *entities_at_000.iter().next().unwrap();

    // despawn the entity at (0, 0, 0) to test index removal
    commands.entity(entity_at_000).despawn();
}

fn check_tick(grid_index: Res<SparseGridIndex>, pos_query: Query<Entity, With<GridPosition>>) {
    // there should be only one entity with a position at (2, 3, 4)
    let pos_entity = pos_query.single().expect("Query should get one entity");

    // this should just be a different way of getting that same entity
    let entities_at_234 = grid_index
        .get(&GridPosition(IVec3::new(2, 3, 4)))
        .expect("Expected entity at (2, 3, 4)");

    assert_eq!(entities_at_234.len(), 1);

    let entity_at_234 = *entities_at_234.iter().next().unwrap();

    assert_eq!(pos_entity, entity_at_234);

    // nothing should be indexed at (0, 0, 0) since that entity was despawned
    assert!(grid_index.get(&GridPosition(IVec3::ZERO)).is_none());

    // nothing should be indexed at (1, 2, 3) since that entity moved
    assert!(grid_index.get(&GridPosition(IVec3::new(1, 2, 3))).is_none());
}
