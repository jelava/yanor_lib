use bevy::prelude::*;
use yanor_core::{
    activity::{Active, Inactive},
    grid::*,
    input::ActiveInputController,
    tick::TickState,
};

use crate::{player::Player, presentation::AssetHandles, step::Step};

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, spawn_cursor).add_systems(
            Update,
            (
                cursor_movement_controls, //.run_if(in_state(TickState::PreTick)),
                cursor_interact_move,     //.run_if(in_state(TickState::PreTick)),
            )
                .run_if(in_state(TickState::PreTick)),
        );
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
    ));
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
    } else {
        return;
    }

    let new_pos: GridPos = cursor_pos + move_dir;

    commands.entity(cursor).insert(new_pos);

    transform.translation = new_pos.into();
}

fn cursor_interact_move(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    cursor_pos: Single<&GridPos, With<Cursor>>,
    active_input_controller: Single<
        (Entity, &GridPos),
        (With<ActiveInputController>, With<Inactive>),
    >,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let &GridPos(cursor_vec) = *cursor_pos;
        let (active_entity, &GridPos(active_vec)) = *active_input_controller;

        let cursor_offset = cursor_vec - active_vec;

        if let Ok(dir) = cursor_offset.try_into() {
            if dir != GridDir::ZERO {
                commands.entity(active_entity).insert(Active(Step(dir)));
            }
        }
    }
}
