use bevy::prelude::*;
use yanor_core::{activity::*, grid::*, stats::*, tick::*};

pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.init_activity::<Step>()
            .add_systems(FixedUpdate, step_tick.run_if(in_state(TickState::Tick)))
            .add_observer(on_step_phase_finished);
    }
}

#[derive(Clone, Copy, Default)]
pub enum AxisDirection {
    #[default]
    Zero,
    Plus,
    Minus,
}

#[derive(Clone, Copy)]
pub struct GridDirection {
    x: AxisDirection,
    y: AxisDirection,
    z: AxisDirection,
}

impl GridDirection {
    pub fn new(x: AxisDirection, y: AxisDirection, z: AxisDirection) -> Self {
        Self { x, y, z }
    }
}

impl From<GridDirection> for IVec3 {
    fn from(value: GridDirection) -> Self {
        use AxisDirection::*;

        let x = match value.x {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        let y = match value.y {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        let z = match value.z {
            Zero => 0,
            Plus => 1,
            Minus => -1,
        };

        IVec3 { x, y, z }
    }
}

pub(crate) struct Step(pub GridDirection);

impl Activity for Step {
    type Phase = StepPhase;

    fn name(&self) -> String {
        "Step".into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([StepPhase::BeginStep, StepPhase::EndStep].into())
    }
}

// temp, hacky
pub const MOVE_SPEED_STAT_ID: StatId = StatId(123);

#[derive(Clone, Debug)]
pub(crate) enum StepPhase {
    BeginStep,
    EndStep,
}

impl ActivityPhase for StepPhase {
    fn name(&self) -> String {
        format!("{self:?}")
    }

    fn duration(&self) -> StatId {
        MOVE_SPEED_STAT_ID // both phases last 5 ticks
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
    trigger: Trigger<FinishActivityPhase<StepPhase>>,
    mut commands: Commands,
    // grid_index: Res<SparseGridIndex>,
    step_query: Query<(&Active<Step>, &GridPosition)>,
) {
    let entity = trigger.target();

    if let Ok((&Active(Step(dir)), &GridPosition(current_pos))) = step_query.get(entity) {
        match trigger.event() {
            FinishActivityPhase(StepPhase::BeginStep) => {
                // TODO: use index to check for collisions!

                commands
                    .entity(entity)
                    .insert(GridPosition(current_pos + IVec3::from(dir)));
            }
            _ => {}
        }
    }
}
