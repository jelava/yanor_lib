use std::collections::VecDeque;

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
// use enum_map::EnumArray;

use crate::{
    input::ActiveInputController,
    stats::{StatBlock, StatId},
    tick::*,
};

pub trait ActivityApp {
    fn init_activity<A: Activity>(&mut self) -> &mut App;
}

// TODO: implement for SubApp and use that implementation for App
impl ActivityApp for App {
    fn init_activity<A: Activity>(&mut self) -> &mut App {
        self.add_systems(OnEnter(TickState::PreTick), finish_pre_tick_if_active::<A>)
            .add_systems(
                FixedUpdate,
                advance_activity_phase_queues::<A>.run_if(in_state(TickState::Tick)),
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
    fn duration(&self) -> StatId;
}

#[derive(Component)]
#[require(StatBlock<u32>)]
pub struct ActivityPhaseQueue<P: ActivityPhase> {
    queue: VecDeque<P>,
    phase_timer: TickTimer,
}

impl<P: ActivityPhase> ActivityPhaseQueue<P> {
    pub fn new(queue: VecDeque<P>) -> Self {
        Self {
            queue,
            phase_timer: TickTimer::new(0), // the actual duration of the timer will be set in advance_activity_phase_queues
        }
    }

    fn peek(&self) -> Option<&P> {
        self.queue.front()
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
            .insert(phase_queue) // TODO: instead of
            .remove::<PendingPreTick>()
            .try_remove::<Inactive>()
            .try_remove::<ActiveInputController>();
    } else {
        warn!("No Active component found in on_insert hook for Active");
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

#[derive(EntityEvent)]
pub struct BeginActivityPhase<P: ActivityPhase> {
    #[event_target]
    entity: Entity,
    pub phase: P,
}

#[derive(EntityEvent)]
pub struct FinishActivityPhase<P: ActivityPhase> {
    #[event_target]
    entity: Entity,
    pub phase: P,
}

fn advance_activity_phase_queues<A: Activity>(
    mut commands: Commands,
    mut queue_query: Query<
        (Entity, &mut ActivityPhaseQueue<A::Phase>, &StatBlock<u32>),
        With<PendingTick>,
    >,
) {
    for (entity, mut queue, stats) in &mut queue_query {
        if let Some(phase) = queue.peek() {
            // Check the actual value of the duration every time, because it may change
            // between ticks (i.e. a speed buff being applied and/or expiring)
            // TODO: change to get() once adjusted stats implemented
            let phase_duration = match stats.get_base(&phase.duration()) {
                Some(&duration) => duration,
                None => {
                    warn!(
                        "StatBlock has no value for the duration stat of the current ActivityPhase, setting duration to 1 tick"
                    );
                    1
                }
            };

            queue.phase_timer.set_duration(phase_duration);

            if queue.phase_timer.tick(1).finished() {
                let (maybe_old_phase, maybe_new_phase) = queue.pop();

                if let Some(old_phase) = maybe_old_phase {
                    commands.trigger(FinishActivityPhase {
                        entity,
                        phase: old_phase,
                    });
                }

                if let Some(new_phase) = maybe_new_phase {
                    commands.trigger(BeginActivityPhase {
                        entity,
                        phase: new_phase.clone(),
                    });

                    queue.phase_timer.reset();
                } else {
                    commands
                        .entity(entity)
                        .remove::<ActivityPhaseQueue<A::Phase>>()
                        .remove::<Active<A>>()
                        .insert(Inactive);
                }
            }
        }

        commands.entity(entity).remove::<PendingTick>();
    }
}
