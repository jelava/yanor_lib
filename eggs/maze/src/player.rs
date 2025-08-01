use bevy::prelude::*;
use yanor_core::{
    activity::*,
    input::ActiveInputController,
    tick::*,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_active_input_controller.run_if(in_state(TickState::PreTick)));
    }
}


#[derive(Component)]
pub struct Player;

enum GridDirection {
    Forward,
    Backward,
    Left,
    Right
}

struct Move(GridDirection);

impl Activity for Move {
    type Phase = MovePhase;

    fn name(&self) -> String {
        String::from("Move")
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([
            MovePhase::BeginStep,
            MovePhase::EndStep,
        ].into())
    }
}

#[derive(Clone)]
enum MovePhase {
    BeginStep,
    EndStep,
}

impl ActivityPhase for MovePhase {
    fn name(&self) -> String {
        todo!()
    }

    fn duration(&self) -> usize {
        todo!()
    }
}

fn process_active_input_controller(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    active_input_controller_query: Single<Entity, (With<ActiveInputController>, With<Idle>)>,
) {
    let entity = *active_input_controller_query;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        commands
            .entity(entity)
            .insert(Active(Move(GridDirection::Forward)));
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        commands
            .entity(entity)
            .insert(Active(Move(GridDirection::Backward)));
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        commands
            .entity(entity)
            .insert(Active(Move(GridDirection::Left)));
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        commands
            .entity(entity)
            .insert(Active(Move(GridDirection::Right)));
    }
}

fn do_move_activity(
    mut commands: Commands,
    player_activity_query: Query<(Entity, &Active<Move>), With<NeedsTick>>,
) {

}
