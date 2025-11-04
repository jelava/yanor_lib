use bevy::prelude::*;
use yanor_core::{activity::*, grid::*, stats::*, tick::*};

use crate::StepLatencyStopwatch;

pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_activity::<Step>()
            .add_systems(FixedUpdate, step_tick.run_if(in_state(TickState::Tick)))
            .add_observer(on_step_phase_finished);
    }
}

pub(crate) struct Step(pub GridDir);

impl Activity for Step {
    type Phase = StepPhase;

    fn name(&self) -> String {
        "Step".into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([StepPhase::BeginStep, StepPhase::EndStep].into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StepPhase {
    BeginStep,
    EndStep,
}

// temp, hacky
pub const MOVE_DURATION_STAT_ID: StatId = StatId(0);

impl ActivityPhase for StepPhase {
    fn name(&self) -> String {
        format!("{self:?}")
    }

    fn duration(&self) -> StatId {
        MOVE_DURATION_STAT_ID
    }
}

fn step_tick(
    mut commands: Commands,
    step_activity_query: Query<Entity, (With<Active<Step>>, With<PendingTick>)>,
) {
    // there's no specific per-tick logic needed for taking a step, so just remove PendingTick
    for entity in &step_activity_query {
        commands.entity(entity).remove::<PendingTick>();
    }

    // TODO: there should be probably be a generic way to indicate that an activity doesn't need
    // per-tick updates to avoid needing to create a bunch of boilerplate systems like this
}

fn on_step_phase_finished(
    trigger: On<FinishActivityPhase<StepPhase>>,
    mut commands: Commands,
    // grid_index: Res<SparseGridIndex>,
    step_query: Query<(&Active<Step>, &GridPos)>,
    mut step_latency_stopwatch: ResMut<StepLatencyStopwatch>,
) {
    let entity = trigger.event_target();

    if let Ok((&Active(Step(dir)), &GridPos(current_pos))) = step_query.get(entity) {
        match trigger.event().phase {
            StepPhase::BeginStep => {
                step_latency_stopwatch.0.pause();
                info!(
                    "step latency: {} s",
                    step_latency_stopwatch.0.elapsed_secs()
                );
                step_latency_stopwatch.0.reset();

                // TODO: use index to check for collisions!

                commands
                    .entity(entity)
                    .insert(GridPos(current_pos + IVec3::from(dir)));
            }
            _ => {}
        }
    }
}
