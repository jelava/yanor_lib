use std::collections::VecDeque;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    PlayerId,
    tick::{Idle, TickState},
};

pub struct InputControllerPlugin;

impl Plugin for InputControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputControllerQueue>()
            .add_systems(OnEnter(TickState::PreTick), queue_input_controllers);
    }
}

#[derive(Resource, Default)]
pub struct InputControllerQueue(VecDeque<Entity>);
// TODO: handle entities in queue being despawned/having InputController component removed?

fn queue_input_controllers(
    mut commands: Commands,
    mut input_queue: ResMut<InputControllerQueue>,
    input_controller_query: Query<(Entity, &PlayerId), (With<InputController>, With<Idle>)>,
) {
    if !input_queue.0.is_empty() {
        warn!("InputControllerQueue was not empty at beginning of PreTick, clearing queue");
        input_queue.0.clear();
    }

    input_controller_query
        .iter()
        .sort_by::<&PlayerId>(|PlayerId(player1), PlayerId(player2)| player1.cmp(player2))
        .for_each(|(entity, player)| {
            info!("adding {player:?} to InputControllerQueue");
            input_queue.0.push_back(entity);
        });

    if let Some(entity) = input_queue.0.pop_front() {
        commands.entity(entity).insert(ActiveInputController);
    }
}

#[derive(Component)]
#[require(Idle)]
pub struct InputController;

// The currently active input controller
#[derive(Component)]
#[component(on_remove = next_active_input_controller)]
pub struct ActiveInputController;

fn next_active_input_controller(mut world: DeferredWorld, _context: HookContext) {
    let mut input_queue = world.resource_mut::<InputControllerQueue>();

    info!("ActiveInputController removed");

    if let Some(next_input_controller_entity) = input_queue.0.pop_front() {
        info!("Found next InputController in queue");

        world
            .commands()
            .entity(next_input_controller_entity)
            .insert(ActiveInputController);
    } else {
        info!("InputControllerQueue is empty");
    }
}
