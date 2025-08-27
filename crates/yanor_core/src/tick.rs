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
/// PostTick states, since they are useful for scheduling most tick-related systems and are
/// somewhat analogous in purpose to Bevy's PreUpdate, Update, and PostUpdate schedules.
///
/// TODO: explain how tick phases progress via marker components for pending phase
///
/// The PresentTick state is for presentation of state changes in between ticks. For a local game
/// client, presentation might consist of playing animations to show any actions that occurred
/// during the tick. For a game running on a server, on the other hand, presentation might instead
/// be sending out updates over the network to clients.
///
/// Making a dedicated state for presentation logic is useful because it allows presentation
/// concerns to be decoupled from the specific details of tick phases (which are primarily useful
/// for Tickable entity logic) without occurring completely independently of the tick loop. This
/// is important, for example, when playing animations to present state changes to a user because
/// if the tick loop simply continues running while previous state changes are animated then the
/// animations will eventually end up falling behind the current state of the game in ways that
/// could be confusing.
#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
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
    mut next_tick_phase: ResMut<NextState<TickState>>,
    last_pending_query: Single<Entity, With<C>>,
) {
    if *last_pending_query == trigger.target() {
        next_tick_phase.set(tick_state.get().next());
        commands.entity(trigger.observer()).despawn();
    } else {
        warn!("Target of trigger is not the same as last remaining enity in pending query");
    }
}
