use bevy::{dev_tools::states::*, prelude::*, state::app::StatesPlugin};

use crate::{Idle, PendingTick};

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StatesPlugin)
            .init_state::<TickState>()
            .add_systems(Startup, start_ticking)
            .add_systems(Update, log_transitions::<TickState>)
            .add_systems(
                FixedUpdate,
                (
                    check_for_idlers.run_if(in_state(TickState::PreTick)),
                    check_for_pending.run_if(in_state(TickState::Tick)),
                ),
            );
            // .add_systems(OnEnter(PostTick), )
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

// #[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct PreTick;
//
// #[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct Tick;
//
// #[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
// pub struct PostTick;

fn start_ticking(mut next_tick_state: ResMut<NextState<TickState>>) {
    info!("starting ticks...");
    next_tick_state.set(TickState::PreTick);
}

fn check_for_idlers(mut next_tick_state: ResMut<NextState<TickState>>, idler_query: Query<&Idle>) {
    if idler_query.is_empty() {
        info!("No idlers, leaving PreTick state");
        next_tick_state.set(TickState::Tick);
    }
}

fn check_for_pending(mut next_tick_state: ResMut<NextState<TickState>>, pending_query: Query<&PendingTick>) {
    if pending_query.is_empty() {
        info!("All pending tick activities processed, leaving Tick state");
        next_tick_state.set(TickState::PostTick);
    }
}
