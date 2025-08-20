pub mod timing;

use bevy::prelude::*;

pub use timing::{TickStopwatch, TickTimer};

pub struct TickPlugin;

// TODO: add optional configuration to determine whether start_ticking() is added to a Startup/PostStartup schedule by the plugin
impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        use TickState::*;

        // tick_state_setup will run after OnExit for the old state and before OnEnter for the new
        // state (see https://docs.rs/bevy/latest/bevy/state/state/enum.StateTransitionSteps.html)
        app.init_state::<TickState>()
            .add_systems(
                OnTransition {
                    exited: NotYetTicking,
                    entered: PreTick,
                },
                tick_state_setup::<PendingPreTick>,
            )
            .add_systems(
                OnTransition {
                    exited: PreTick,
                    entered: Tick,
                },
                tick_state_setup::<PendingTick>,
            )
            .add_systems(
                OnTransition {
                    exited: Tick,
                    entered: PostTick,
                },
                tick_state_setup::<PendingPostTick>,
            )
            .add_systems(
                OnTransition {
                    exited: PostTick,
                    entered: PreTick,
                },
                tick_state_setup::<PendingPreTick>,
            );
    }
}

/// A tick is a single discrete step forward in the game simulation, and each TickState represents
/// a different stage of the tick. The most important states here are the PreTick, Tick, and
/// PostTick states, since they are the actual phases of the tick and are somewhat analogous in
/// purpose to Bevy's PreUpdate, Update, and PostUpdate schedules.
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum TickState {
    #[default]
    NotYetTicking,
    PreTick,
    Tick,
    PostTick,
}

impl TickState {
    fn next(&self) -> Self {
        use TickState::*;

        match self {
            NotYetTicking => PreTick,
            PreTick => Tick,
            Tick => PostTick,
            PostTick => PreTick,
        }
    }
}

// TODO? this name is kinda silly, maybe change it???
#[derive(Component, Default)]
pub struct Tickable;

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct PendingPreTick;

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct PendingTick;

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct PendingPostTick;

pub fn start_ticking(mut next_tick_state: ResMut<NextState<TickState>>) {
    next_tick_state.set(TickState::PreTick);
}

fn tick_state_setup<C: Component + Default>(
    mut commands: Commands,
    tickable_query: Query<Entity, With<Tickable>>,
) {
    for entity in &tickable_query {
        commands.entity(entity).insert(C::default());
    }

    commands.add_observer(on_pending_component_remove::<C>);
}

fn on_pending_component_remove<C: Component>(
    trigger: Trigger<OnRemove, C>,
    mut commands: Commands,
    tick_state: Res<State<TickState>>,
    mut next_tick_state: ResMut<NextState<TickState>>,
    last_pending_query: Single<Entity, With<C>>,
) {
    if *last_pending_query == trigger.target() {
        next_tick_state.set(tick_state.get().next());
        commands.entity(trigger.observer()).despawn();
    } else {
        warn!("Target of trigger is not the same as last remaining enity in pending query");
    }
}
