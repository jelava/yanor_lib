use bevy::prelude::*;

pub struct TickPlugin;

// TODO: add optional configuration to determine whether start_ticking() is added to a Startup/PostStartup schedule by the plugin
impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        use TickState::*;

        // tick_state_setup needs to run *before* any systems scheduled for OnEnter<TickState>, so
        // run it OnExit of the old state instead (OnExit runs before OnEnter, see
        // https://docs.rs/bevy/latest/bevy/state/state/enum.StateTransitionSteps.html)
        app.init_state::<TickState>()
            .add_systems(OnExit(NotYetTicking), tick_state_setup::<PendingPreTick>)
            .add_systems(OnExit(PreTick), tick_state_setup::<PendingTick>)
            .add_systems(OnExit(Tick), tick_state_setup::<PendingPostTick>)
            .add_systems(OnExit(PostTick), tick_state_setup::<PendingPreTick>);
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
        info!("target of trigger is not last remaining pending?");
    }
}
