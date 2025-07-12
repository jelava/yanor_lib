use bevy::{
    dev_tools::states::*,
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
    state::app::StatesPlugin,
};

use crate::{PlayerId, activity::*};

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatesPlugin)
            .init_state::<TickState>()
            // .add_systems(Startup, start_ticking)
            .add_systems(Update, log_transitions::<TickState>)
            .add_systems(
                FixedUpdate,
                (
                    check_for_idlers.run_if(in_state(TickState::PreTick)),
                    check_for_pending.run_if(in_state(TickState::Tick)),
                    advance_activity_phase_queues.run_if(in_state(TickState::PostTick)),
                ),
            );
    }
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum TickState {
    #[default]
    NotYetTicking,
    PreTick,
    Tick,
    PostTick,
}

// pre-tick: Idle -> PendingTick
// tick: PendingTick -> no PendingTick
// post-tick: not pending or idle -> PendingTick (activity not done) or Idle (activity done)

#[derive(Component, Default)]
#[component(on_add = remove_others::<NeedsTick, TickDone>)] //remove1)]
pub struct Idle;

#[derive(Component, Default)]
#[component(on_add = remove_others::<Idle, TickDone>)]
pub struct NeedsTick;

#[derive(Component, Default)]
#[component(on_add = remove_others::<Idle, NeedsTick>)]
pub struct TickDone;

// this is hacky and not generalizable but good enough for now
fn remove_others<C1: Component, C2: Component>(
    mut world: DeferredWorld,
    HookContext { entity, .. }: HookContext,
) {
    world
        .commands()
        .entity(entity)
        .remove::<C1>()
        .remove::<C2>();
}

// fn remove1(
//     mut world: DeferredWorld,
//     context: HookContext,
// ) {
//     remove_others<NeedsTick, TickDone>(world, context);
// }

pub fn start_ticking(mut next_tick_state: ResMut<NextState<TickState>>) {
    info!("starting ticks...");
    next_tick_state.set(TickState::PreTick);
}

fn check_for_idlers(mut next_tick_state: ResMut<NextState<TickState>>, idler_query: Query<&Idle>) {
    if idler_query.is_empty() {
        info!("No idlers, leaving PreTick state");
        next_tick_state.set(TickState::Tick);
    } else {
        info!("PreTick: waiting for idlers...");
    }
}

fn check_for_pending(
    mut next_tick_state: ResMut<NextState<TickState>>,
    pending_query: Query<&NeedsTick>,
) {
    if pending_query.is_empty() {
        info!("All pending tick activities processed, leaving Tick state");
        next_tick_state.set(TickState::PostTick);
    } else {
        info!("Tick: some entities still pending...");
    }
}

fn advance_activity_phase_queues(
    mut commands: Commands,
    mut next_tick_state: ResMut<NextState<TickState>>,
    mut queue_query: Query<(Entity, &PlayerId, &mut ActivityPhaseQueue), With<TickDone>>,
) {
    // use ActivityTickUpdate::*;

    let mut query_empty = true;

    for (entity, player, mut queue) in &mut queue_query {
        info!("advancing activity for {player:?}");

        let mut entity_commands = commands.entity(entity);

        query_empty = false;

        info!(
            "before phase: {}",
            queue.peek().map_or("None", |phase| &phase.name)
        );
        info!("before ticks: {}", queue.ticks_to_next_phase);

        if queue.ticks_to_next_phase > 1 {
            info!("continuing current phase");
            queue.ticks_to_next_phase -= 1;
            entity_commands.insert(NeedsTick);
        } else if let Some(phase) = queue.pop() {
            info!("entering phase: {}", phase.name);

            // TODO: fire an event on phase change?
            queue.ticks_to_next_phase = phase.duration;
            entity_commands.insert(NeedsTick);
        } else {
            info!("no phases left, activity over");
            queue.ticks_to_next_phase = 0;
            entity_commands.insert(Idle);
        }

        info!(
            "after phase: {}",
            queue.peek().map_or("None", |phase| &phase.name)
        );
        info!("after ticks: {}", queue.ticks_to_next_phase);
    }

    if query_empty {
        info!("All activities updated, leaving PostTick state and beginning next tick");
        next_tick_state.set(TickState::PreTick);
    }
}
