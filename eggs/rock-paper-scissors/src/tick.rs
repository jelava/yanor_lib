use bevy::{
    dev_tools::states::*,
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<TickState>()
            .add_systems(Update, log_transitions::<TickState>)
            .add_systems(
                FixedPostUpdate,
                (
                    check_for_idlers.run_if(in_state(TickState::PreTick)),
                    check_for_pending.run_if(in_state(TickState::Tick)),
                    check_for_tick_done.run_if(in_state(TickState::PostTick)),
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

#[derive(Component, Default)]
#[component(on_add = remove_others::<NeedsTick, TickDone>)]
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
        .try_remove::<C1>()
        .try_remove::<C2>();
}

pub fn start_ticking(mut next_tick_state: ResMut<NextState<TickState>>) {
    next_tick_state.set(TickState::PreTick);
}

fn check_for_idlers(mut next_tick_state: ResMut<NextState<TickState>>, idler_query: Query<&Idle>) {
    if idler_query.is_empty() {
        next_tick_state.set(TickState::Tick);
    }
}

fn check_for_pending(
    mut next_tick_state: ResMut<NextState<TickState>>,
    pending_query: Query<&NeedsTick>,
) {
    if pending_query.is_empty() {
        next_tick_state.set(TickState::PostTick);
    }
}

fn check_for_tick_done(
    mut next_tick_state: ResMut<NextState<TickState>>,
    tick_done_query: Query<&TickDone>,
) {
    if tick_done_query.is_empty() {
        next_tick_state.set(TickState::PreTick);
    }
}
