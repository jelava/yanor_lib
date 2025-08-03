use bevy::prelude::*;
use yanor_core::{
    activity::*,
    grid::{GridPosition, SparseGridIndex},
    input::ActiveInputController,
    tick::*,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_activity::<Step>()
            .add_systems(
                FixedUpdate,
                process_active_input_controller.run_if(in_state(TickState::PreTick)),
            )
            .add_systems(FixedUpdate, step_tick.run_if(in_state(TickState::Tick)))
            .add_observer(on_step_phase_finished);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Clone, Copy, Default)]
pub enum AxisDirection {
    #[default]
    Zero,
    Plus,
    Minus,
}

// TODO: fill out with more directions
#[derive(Clone, Copy)]
pub struct GridDirection {
    x: AxisDirection,
    y: AxisDirection,
    z: AxisDirection,
}

impl GridDirection {
    fn new(x: AxisDirection, y: AxisDirection, z: AxisDirection) -> Self {
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

pub(crate) struct Step(GridDirection);

impl Activity for Step {
    type Phase = StepPhase;

    fn name(&self) -> String {
        "Step".into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([StepPhase::BeginStep, StepPhase::EndStep].into())
    }
}

#[derive(Clone, Debug)]
pub(crate) enum StepPhase {
    BeginStep,
    EndStep,
}

impl ActivityPhase for StepPhase {
    fn name(&self) -> String {
        format!("{self:?}")
    }

    fn duration(&self) -> usize {
        5 // both phases last 5 ticks
    }
}

fn process_active_input_controller(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    active_input_controller_query: Single<Entity, (With<ActiveInputController>, With<Inactive>)>,
) {
    use AxisDirection::*;

    let entity = *active_input_controller_query;

    // TODO: check grid index if move is even possible

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Zero, Zero, Plus))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Zero, Zero, Minus))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Plus, Zero, Zero))));
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(Minus, Zero, Zero))));
    }
}

fn step_tick(
    mut commands: Commands,
    player_activity_query: Query<Entity, (With<Active<Step>>, With<PendingTick>)>,
) {
    // there's no specific per-tick logic needed for taking a step, so just remove PendingTick
    for entity in &player_activity_query {
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
