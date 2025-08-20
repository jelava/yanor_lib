use bevy::prelude::*;
use yanor_core::{
    activity::*,
    // grid::{GridPosition, SparseGridIndex},
    input::ActiveInputController,
    tick::*,
};

use crate::step::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            process_active_input_controller.run_if(in_state(TickState::PreTick)),
        );
    }
}

#[derive(Component)]
pub struct Player;

fn process_active_input_controller(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    active_input_controller_query: Single<Entity, (With<ActiveInputController>, With<Inactive>)>,
) {
    use AxisDirection::*;

    let entity = *active_input_controller_query;

    // TODO: check grid index if move is even possible


    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        info!("the fuck is going on");

        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Zero, Zero, Plus))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Zero, Zero, Minus))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Plus, Zero, Zero))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Minus, Zero, Zero))));
    }
}

