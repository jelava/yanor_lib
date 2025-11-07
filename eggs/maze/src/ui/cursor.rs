use bevy::prelude::*;
use yanor_core::{
    activity::{Active, Inactive},
    grid::*,
    index::ComponentIndex,
    input::ActiveInputController,
    tick::TickState,
};

use crate::{
    Block,
    collision::Collider,
    player::Player,
    presentation::AssetHandles,
    step::Step,
    ui::*
};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostStartup, spawn_cursor)
            .add_systems(
                Update,
                (cursor_movement_controls, cursor_interact).run_if(in_state(TickState::PreTick)
            ))
            .add_systems(OnEnter(TickState::PreTick), update_cursor_selection_list);
    }
}

#[derive(Component)]
#[require(GridPos)]
struct Cursor;

fn spawn_cursor(
    mut commands: Commands,
    player_pos: Single<&GridPos, With<Player>>,
    asset_handles: Res<AssetHandles>,
) {
    commands.spawn((
        Cursor,
        **player_pos,
        Transform::from_translation((**player_pos).into()),
        Mesh3d(asset_handles.block_mesh_handle.clone()),
        MeshMaterial3d(asset_handles.cursor_material_handle.clone()),
    )).observe(update_cursor_selection_list_on_move);
}

fn update_cursor_selection_list(
    mut commands: Commands,
    grid_index: Res<SparseGridIndex>,
    cursor_pos: Single<&GridPos, With<Cursor>>,
    selection_list_entity: Single<Entity, With<CursorSelectionList>>,
    preview_info_query: Query<&UiPreviewKind>,
) {
    info!("update cursor list");

    commands
        .entity(*selection_list_entity)
        .despawn_children();

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
    update_cursor_selection_list(commands, grid_index, cursor_pos, selection_list_entity, preview_info_query);
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
    collider_query: Query<&Collider>,
    block_query: Query<&Block>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let &GridPos(cursor_vec) = *cursor_pos;
        let (active_entity, &GridPos(active_vec)) = *active_input_controller;

        let cursor_offset = cursor_vec - active_vec;

        if let Ok(dir) = cursor_offset.try_into() {
            if dir != GridDir::ZERO {
                let cursor_pos_empty = match grid_index.get(*cursor_pos) {
                    Some(entities) => entities
                        .iter()
                        .all(|&entity| !collider_query.contains(entity)),
                    None => true,
                };

                let below_cursor = **cursor_pos + GridDir::NEG_Y;

                let is_walkable = match grid_index.get(&below_cursor) {
                    Some(entities) => {
                        entities.iter().any(|&entity| block_query.contains(entity)) // TODO: more general "traversable" property?
                    }
                    None => false, // TODO: what if cursor-controlled entity is flying/floating?
                };

                if cursor_pos_empty && is_walkable {
                    commands.entity(active_entity).insert(Active(Step(dir)));
                }
            }
        }
    }
}
