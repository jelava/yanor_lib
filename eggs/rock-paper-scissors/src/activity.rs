use std::collections::VecDeque;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::tick::{Idle, NeedsTick};

pub trait Activity: Send + Sync + 'static {
    fn name(&self) -> String;
    fn phase_queue(&self) -> ActivityPhaseQueue;
}

#[derive(Clone)]
pub struct ActivityPhase {
    pub name: String,
    pub duration: usize,
}

#[derive(Component)]
pub struct ActivityPhaseQueue {
    queue: VecDeque<ActivityPhase>,
    pub ticks_to_next_phase: usize,
}

impl ActivityPhaseQueue {
    pub fn new(queue: VecDeque<ActivityPhase>) -> Self {
        Self {
            ticks_to_next_phase: queue.front().map_or(0, |phase| phase.duration),
            queue,
        }
    }

    pub fn peek(&self) -> Option<&ActivityPhase> {
        self.queue.front()
    }

    // Unlike VecDeque, this will pop off the first element and return the *new* front of the queue,
    // (i.e. the next phase in the queue) rather than whatever was popped.
    pub fn pop(&mut self) -> Option<&ActivityPhase> {
        self.queue.pop_front();
        self.queue.front()
    }
}

#[derive(Component)]
#[require(NeedsTick)]
#[component(on_add = init_phase_queue::<A>)]
pub struct CurrentActivity<A: Activity>(pub A);

fn init_phase_queue<A: Activity>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    let phase_queue = world
        .get::<CurrentActivity<A>>(entity)
        .unwrap() // TODO: don't do this
        .0
        .phase_queue();

    world
        .commands()
        .entity(entity)
        .insert(phase_queue)
        .observe(remove_activity_when_idle::<A>);
}

fn remove_activity_when_idle<A: Activity>(trigger: Trigger<OnAdd, Idle>, mut commands: Commands) {
    info!("Idle component added, removing CurrentActivity");

    commands
        .entity(trigger.observer())
        .remove::<CurrentActivity<A>>();
}
