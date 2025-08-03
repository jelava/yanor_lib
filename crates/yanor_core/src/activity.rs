use std::collections::VecDeque;

use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::{input::ActiveInputController, tick::*};

pub trait ActivityApp {
    fn init_activity<A: Activity>(&mut self) -> &mut App;
}

// TODO: implement for SubApp and use that implementation for App
impl ActivityApp for App {
    fn init_activity<A: Activity>(&mut self) -> &mut App {
        self.add_systems(OnEnter(TickState::PreTick), finish_pre_tick_if_active::<A>)
            .add_systems(
                OnEnter(TickState::PostTick),
                advance_activity_phase_queues::<A>,
            )
    }
}

pub trait Activity: Send + Sync + 'static {
    type Phase: ActivityPhase;

    fn name(&self) -> String;
    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase>;
}

pub trait ActivityPhase: Clone + Send + Sync {
    fn name(&self) -> String;
    fn duration(&self) -> usize;
}

#[derive(Component)]
pub struct ActivityPhaseQueue<P: ActivityPhase> {
    queue: VecDeque<P>,
    pub ticks_to_next_phase: usize,
}

impl<P: ActivityPhase> ActivityPhaseQueue<P> {
    pub fn new(queue: VecDeque<P>) -> Self {
        Self {
            ticks_to_next_phase: queue.front().map_or(0, |phase| phase.duration()),
            queue,
        }
    }

    // Return both the previous (just removed) phase and the new current phase
    fn pop(&mut self) -> (Option<P>, Option<&P>) {
        (self.queue.pop_front(), self.queue.front())
    }
}

/// Indicates that the component is not currently doing any Activity
#[derive(Component, Default)]
#[require(Tickable)]
#[component(storage = "SparseSet")]
pub struct Inactive;

/// Add this component to an entity to indicate that the entity is currently performing that activity
#[derive(Component)]
#[component(
    immutable,
    storage = "SparseSet",
    on_insert = init_phase_queue::<A>,
)]
pub struct Active<A: Activity>(pub A);

fn init_phase_queue<A: Activity>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    if let Some(Active(activity)) = world.get::<Active<A>>(entity) {
        let phase_queue = activity.phase_queue();

        world
            .commands()
            .entity(entity)
            .insert(phase_queue)
            .remove::<PendingPreTick>()
            .try_remove::<Inactive>()
            .try_remove::<ActiveInputController>();
    } else {
        warn!("No CurrentActivity found in on_insert hook for CurrentActivity");
    }
}

fn finish_pre_tick_if_active<A: Activity>(
    mut commands: Commands,
    pending_query: Query<Entity, (With<PendingPreTick>, With<Active<A>>)>,
) {
    for entity in pending_query {
        commands.entity(entity).remove::<PendingPreTick>();
    }
}

#[derive(Event)]
pub struct BeginActivityPhase<P: ActivityPhase>(pub P);

#[derive(Event)]
pub struct FinishActivityPhase<P: ActivityPhase>(pub P);

fn advance_activity_phase_queues<A: Activity>(
    mut commands: Commands,
    mut queue_query: Query<(Entity, &mut ActivityPhaseQueue<A::Phase>)>,
) {
    for (entity, mut queue) in &mut queue_query {
        if queue.ticks_to_next_phase > 1 {
            queue.ticks_to_next_phase -= 1;
        } else {
            // phase change
            let (maybe_old_phase, maybe_new_phase) = queue.pop();

            if let Some(old_phase) = maybe_old_phase {
                commands.trigger_targets(FinishActivityPhase(old_phase), entity);
            }

            if let Some(new_phase) = maybe_new_phase {
                commands.trigger_targets(BeginActivityPhase(new_phase.clone()), entity);

                queue.ticks_to_next_phase = new_phase.duration();
            } else {
                commands
                    .entity(entity)
                    .remove::<ActivityPhaseQueue<A::Phase>>()
                    .remove::<Active<A>>()
                    .insert(Inactive);
            }
        }

        commands.entity(entity).remove::<PendingPostTick>();
    }
}
