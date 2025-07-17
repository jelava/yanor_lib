use std::collections::VecDeque;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{
    input::ActiveInputController,
    tick::{Idle, NeedsTick, TickDone, TickState},
};

pub struct ActivityPlugin;

impl Plugin for ActivityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            advance_activity_phase_queues.run_if(in_state(TickState::PostTick)),
        );
    }
}

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

    // Unlike VecDeque, this will pop off the first element and return the *new* front of the queue,
    // (i.e. the next phase in the queue) rather than whatever was popped.
    fn pop(&mut self) -> Option<&ActivityPhase> {
        self.queue.pop_front();
        self.queue.front()
    }
}

fn advance_activity_phase_queues(
    mut commands: Commands,
    mut queue_query: Query<(Entity, &mut ActivityPhaseQueue), With<TickDone>>,
) {
    for (entity, mut queue) in &mut queue_query {
        let mut entity_commands = commands.entity(entity);

        if queue.ticks_to_next_phase > 1 {
            queue.ticks_to_next_phase -= 1;
            entity_commands.insert(NeedsTick);
        } else if let Some(phase) = queue.pop() {
            // TODO: fire an event on phase change?
            queue.ticks_to_next_phase = phase.duration;
            entity_commands.insert(NeedsTick);
        } else {
            queue.ticks_to_next_phase = 0;
            entity_commands.insert(Idle);
        }
    }
}

#[derive(Component)]
#[require(NeedsTick)]
#[component(
    immutable,
    on_insert = init_phase_queue::<A>,
)]
pub struct CurrentActivity<A: Activity>(pub A); // TODO: SparseSet storage?

fn init_phase_queue<A: Activity>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(CurrentActivity(activity)) = world.get::<CurrentActivity<A>>(entity) {
        let phase_queue = activity.phase_queue();

        world
            .commands()
            .entity(entity)
            .insert(phase_queue)
            .try_remove::<ActiveInputController>()
            .observe(remove_activity_when_idle::<A>);
    } else {
        warn!("No CurrentActivity found in on_insert hook for CurrentActivity");
    }
}

fn remove_activity_when_idle<A: Activity>(trigger: Trigger<OnAdd, Idle>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .remove::<CurrentActivity<A>>();

    commands.entity(trigger.observer()).despawn();
}
