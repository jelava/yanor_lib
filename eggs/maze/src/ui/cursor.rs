use bevy::{platform::collections::HashSet, prelude::*};
use yanor_core::{
    activity::{Active, Inactive},
    grid::*,
    index::ComponentIndex,
    input::ActiveInputController,
    tick::TickState,
};

use crate::{
    Block, Stairs,
    collision::{Collider, ColliderShape},
    player::Player,
    presentation::AssetHandles,
    step::Step,
    ui::*,
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_cursor)
            .add_systems(
                Update,
                (cursor_movement_controls, cursor_interact).run_if(in_state(TickState::PreTick)),
            )
            .add_systems(OnEnter(TickState::PreTick), update_cursor_selection_list);
    }
}

#[derive(Component)]
#[require(GridPos)]
struct Cursor;

#[derive(Component)]
struct CursorSelection;

fn spawn_cursor(
    mut commands: Commands,
    player_pos: Single<&GridPos, With<Player>>,
    asset_handles: Res<AssetHandles>,
) {
    commands
        .spawn((
            Cursor,
            **player_pos,
            Transform::from_translation((**player_pos).into()),
            Mesh3d(asset_handles.cube_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.cursor_material_handle.clone()),
        ))
        .observe(update_cursor_selection_list_on_move);
}

fn update_cursor_selection_list(
    mut commands: Commands,
    grid_index: Res<SparseGridIndex>,
    cursor_pos: Single<&GridPos, With<Cursor>>,
    selection_list_entity: Single<Entity, With<CursorSelectionList>>,
    preview_info_query: Query<&UiPreviewKind>,
) {
    info!("update cursor list");

    commands.entity(*selection_list_entity).despawn_children();

    if let Some(entities) = grid_index.get(*cursor_pos) {
        commands
            .entity(*selection_list_entity)
            .with_children(|preview_list| {
                for &entity in entities {
                    if let Ok(preview_kind) = preview_info_query.get(entity) {
                        preview_list.spawn(preview_kind.into_bundle());
                    }
                }
            });
    }
}

fn update_cursor_selection_list_on_move(
    _trigger: On<Insert, GridPos>,
    commands: Commands,
    grid_index: Res<SparseGridIndex>,
    cursor_pos: Single<&GridPos, With<Cursor>>,
    selection_list_entity: Single<Entity, With<CursorSelectionList>>,
    preview_info_query: Query<&UiPreviewKind>,
) {
    update_cursor_selection_list(
        commands,
        grid_index,
        cursor_pos,
        selection_list_entity,
        preview_info_query,
    );
}

fn cursor_movement_controls(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_info: Single<(Entity, &GridPos, &mut Transform), With<Cursor>>,
) {
    use AxisDir::*;

    let (cursor, &cursor_pos, mut transform) = cursor_info.into_inner();
    let move_dir;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        move_dir = GridDir::new(Zero, Zero, Plus);
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        move_dir = GridDir::new(Zero, Zero, Minus);
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        move_dir = GridDir::new(Plus, Zero, Zero);
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        move_dir = GridDir::new(Minus, Zero, Zero);
    } else if keyboard_input.just_pressed(KeyCode::PageDown) {
        move_dir = GridDir::new(Zero, Minus, Zero);
    } else if keyboard_input.just_pressed(KeyCode::PageUp) {
        move_dir = GridDir::new(Zero, Plus, Zero);
    } else {
        return;
    }

    let new_pos: GridPos = cursor_pos + move_dir;
    commands.entity(cursor).insert(new_pos);
    transform.translation = new_pos.into();
}

fn cursor_interact(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid_index: Res<SparseGridIndex>,
    cursor_pos: Single<&GridPos, With<Cursor>>,
    active_input_controller: Single<
        (Entity, &GridPos),
        (With<ActiveInputController>, With<Inactive>),
    >,
    collider_query: Query<(&Collider, Option<&XzPlaneOrientation>)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let &GridPos(cursor_vec) = *cursor_pos;
        let (active_entity, &GridPos(active_vec)) = *active_input_controller;

        let cursor_offset = cursor_vec - active_vec;

        if let Ok(step_dir) = cursor_offset.try_into() {
            if step_dir != GridDir::ZERO {
                let cursor_pos_occupied = match grid_index.get_from_query(*cursor_pos, collider_query) {
                    Some(entities) => !entities.is_empty(),
                    None => false,
                };

                if cursor_pos_occupied {
                    return;
                }

                // TODO: eventually account for orientation of which surface is stood/climbed on
                let below_cursor = **cursor_pos + GridDir::NEG_Y;

                // TODO: what if cursor-controlled entity is flying/floating? (walkability no longer matters)
                let is_unwalkable = match grid_index.get(&below_cursor) {
                    Some(entities) => {
                        entities
                            .iter()
                            .all(|&entity| match collider_query.get(entity) {
                                Ok((Collider { traversable, .. }, _)) => !*traversable,
                                Err(_) => true,
                            })
                    }
                    None => true,
                };

                if is_unwalkable {
                    return;
                }

                let mut adjacent_cells = HashSet::with_capacity(6);

                // TODO: refactor!
                for x_mul in 0..=1 {
                    for y_mul in 0..=1 {
                        for z_mul in 0..=1 {
                            let cell_offset = ivec3(
                                x_mul * cursor_offset.x,
                                y_mul * cursor_offset.y,
                                z_mul * cursor_offset.z,
                            );
                            let cell_vec = active_vec + cell_offset;

                            if cell_vec != active_vec
                                && cell_vec != cursor_vec
                                && adjacent_cells.insert(cell_vec)
                            {
                                //check if the step past an adjacent cell is blocked by a collider
                                let step_blocked = match grid_index.get(&GridPos(cell_vec)) {
                                    Some(entities) => entities
                                        .iter()
                                        .any(|&entity| {
                                            if let Ok((Collider { shape, .. }, maybe_orientation)) = collider_query.get(entity) {
                                                match shape {
                                                    ColliderShape::Block => true,
                                                    ColliderShape::Slope => {
                                                        use XzPlaneOrientation::*;
                                                        // TODO: this should obviously not be hardcoded
                                                        // let orientation = FacingZ;

                                                        let slope_vec = match maybe_orientation {
                                                            Some(FacingZ) => ivec3(0, 1, 1),
                                                            Some(FacingNegZ) => ivec3(0, 1, -1),
                                                            Some(FacingX) => ivec3(1, 1, 0),
                                                            Some(FacingNegX) => ivec3(-1, 1, 0),
                                                            None => {
                                                                warn!("Colliders with Slope shape should have XzPlaneOrientation");
                                                                IVec3::ZERO
                                                            }
                                                        };

                                                        cursor_offset != slope_vec && cursor_offset != -slope_vec
                                                    },
                                                }
                                            } else {
                                                false
                                            }
                                        }),
                                    None => false,
                                };

                                if step_blocked {
                                    return;
                                }
                            }
                        }
                    }
                }

                commands
                    .entity(active_entity)
                    .insert(Active(Step(step_dir)));
            }
        }
    }
}
