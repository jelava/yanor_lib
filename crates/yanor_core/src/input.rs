use std::collections::VecDeque;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::tick::{Idle, TickState};

pub struct InputControllerPlugin;

impl Plugin for InputControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputControllerQueue>()
            .add_systems(OnEnter(TickState::PreTick), queue_input_controllers);
    }
}

#[derive(Component)]
#[require(Idle)]
pub struct InputController {
    pub queue_position: usize,
}

#[derive(Resource, Default)]
pub struct InputControllerQueue(VecDeque<Entity>);
// TODO: handle entities in queue being despawned/having InputController component removed?

fn queue_input_controllers(
    mut commands: Commands,
    mut input_queue: ResMut<InputControllerQueue>,
    input_controller_query: Query<(Entity, &InputController), With<Idle>>,
) {
    if !input_queue.0.is_empty() {
        warn!("InputControllerQueue was not empty at beginning of PreTick, clearing queue");
        input_queue.0.clear();
    }

    input_controller_query
        .iter()
        .sort_by::<&InputController>(|controller1, controller2| {
            controller1.queue_position.cmp(&controller2.queue_position)
        })
        .for_each(|(entity, _)| input_queue.0.push_back(entity));

    if let Some(entity) = input_queue.0.pop_front() {
        commands.entity(entity).insert(ActiveInputController);
    }
}

// The currently active input controller
#[derive(Component)]
#[component(
    storage = "SparseSet",
    on_remove = next_active_input_controller,
)]
pub struct ActiveInputController;

fn next_active_input_controller(mut world: DeferredWorld, _context: HookContext) {
    let mut input_queue = world.resource_mut::<InputControllerQueue>();

    if let Some(next_input_controller_entity) = input_queue.0.pop_front() {
        world
            .commands()
            .entity(next_input_controller_entity)
            .insert(ActiveInputController);
    } //else {
    // TODO: fire an event when InputControllerQueue is empty?
    //}
}
