mod common;

use common::*;

use bevy::prelude::*;
use yanor_core::{activity::*, stats::*, tick::*};

#[derive(Resource, Default)]
struct TickCounter(u32);

#[derive(Component)]
#[require(Inactive)]
struct SimpleController;

const TEST_STAT_1: StatId = StatId(0);
const TEST_STAT_2: StatId = StatId(1);
const TEST_STAT_3: StatId = StatId(2);

struct SimpleActivity;

impl Activity for SimpleActivity {
    type Phase = SimpleActivityPhase;

    fn name(&self) -> String {
        "Simple activity".into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        use SimpleActivityPhase::*;
        ActivityPhaseQueue::new([Phase1, Phase2, Phase3].into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SimpleActivityPhase {
    Phase1,
    Phase2,
    Phase3,
}

impl ActivityPhase for SimpleActivityPhase {
    fn name(&self) -> String {
        "bleh".into()
    }

    fn duration(&self) -> StatId {
        use SimpleActivityPhase::*;

        match self {
            Phase1 => TEST_STAT_1,
            Phase2 => TEST_STAT_2,
            Phase3 => TEST_STAT_3,
        }
    }
}

#[test]
fn single_entity() {
    use TickState::*;

    App::new()
        .add_plugins(BaseTestPlugins)
        .add_plugins(TickPlugin)
        .init_activity::<SimpleActivity>()
        .init_resource::<TickCounter>()
        .add_systems(Startup, (spawn_active_entity, start_ticking).chain())
        .add_systems(
            FixedUpdate,
            (
                process_simple_controllers.run_if(in_state(PreTick)),
                post_tick.run_if(in_state(PostTick)),
                end_test.run_if(|counter: Res<TickCounter>| counter.0 >= 7),
            ),
        )
        .add_systems(OnEnter(PreTick), increment_tick_counter)
        .add_observer(on_phase_finish)
        .run();

    fn spawn_active_entity(mut commands: Commands) {
        commands.spawn((
            SimpleController,
            StatBlock::new(&[(TEST_STAT_1, 1u32), (TEST_STAT_2, 2), (TEST_STAT_3, 3)]),
        ));
    }

    fn on_phase_finish(
        trigger: On<FinishActivityPhase<SimpleActivityPhase>>,
        counter: Res<TickCounter>,
    ) {
        use SimpleActivityPhase::*;

        match trigger.event().phase {
            Phase1 => assert_eq!(counter.0, 1),
            Phase2 => assert_eq!(counter.0, 3),
            Phase3 => assert_eq!(counter.0, 6),
        }
    }
}

fn process_simple_controllers(
    mut commands: Commands,
    controller_query: Query<Entity, (With<SimpleController>, With<Inactive>, With<PendingPreTick>)>,
) {
    for entity in &controller_query {
        commands.entity(entity).insert(Active(SimpleActivity));
    }
}

fn post_tick(mut commands: Commands, post_tick_query: Query<Entity, With<PendingPostTick>>) {
    for entity in &post_tick_query {
        commands.entity(entity).remove::<PendingPostTick>();
    }
}

fn increment_tick_counter(mut counter: ResMut<TickCounter>) {
    counter.0 += 1;
}

#[test]
fn multi_entity() {
    use TickState::*;

    App::new()
        .add_plugins(BaseTestPlugins)
        .add_plugins(TickPlugin)
        .init_activity::<SimpleActivity>()
        .init_resource::<TickCounter>()
        .add_systems(Startup, (spawn_active_entities, start_ticking).chain())
        .add_systems(
            FixedUpdate,
            (
                process_simple_controllers.run_if(in_state(PreTick)),
                post_tick.run_if(in_state(PostTick)),
                end_test.run_if(|counter: Res<TickCounter>| counter.0 >= 10),
            ),
        )
        .add_systems(OnEnter(PreTick), increment_tick_counter)
        .run();

    fn spawn_active_entities(mut commands: Commands) {
        commands
            .spawn((
                SimpleController,
                StatBlock::new(&[(TEST_STAT_1, 1u32), (TEST_STAT_2, 1), (TEST_STAT_3, 1)]),
            ))
            .observe(on_phase_finish_1);

        commands
            .spawn((
                SimpleController,
                StatBlock::new(&[(TEST_STAT_1, 2u32), (TEST_STAT_2, 2), (TEST_STAT_3, 2)]),
            ))
            .observe(on_phase_finish_2);

        commands
            .spawn((
                SimpleController,
                StatBlock::new(&[(TEST_STAT_1, 3u32), (TEST_STAT_2, 3), (TEST_STAT_3, 3)]),
            ))
            .observe(on_phase_finish_3);
    }

    fn on_phase_finish_1(
        trigger: On<FinishActivityPhase<SimpleActivityPhase>>,
        counter: Res<TickCounter>,
    ) {
        use SimpleActivityPhase::*;

        let expected_phase = match counter.0 {
            1 | 4 | 7 => Some(Phase1),
            2 | 5 | 8 => Some(Phase2),
            3 | 6 | 9 => Some(Phase3),
            _ => None,
        };

        if let Some(phase) = expected_phase {
            assert_eq!(trigger.event().phase, phase);
        }
    }

    fn on_phase_finish_2(
        trigger: On<FinishActivityPhase<SimpleActivityPhase>>,
        counter: Res<TickCounter>,
    ) {
        use SimpleActivityPhase::*;

        let expected_phase = match counter.0 {
            2 | 8 => Some(Phase1),
            4 => Some(Phase2),
            6 => Some(Phase3),
            _ => None,
        };

        if let Some(phase) = expected_phase {
            assert_eq!(trigger.event().phase, phase);
        }
    }

    fn on_phase_finish_3(
        trigger: On<FinishActivityPhase<SimpleActivityPhase>>,
        counter: Res<TickCounter>,
    ) {
        use SimpleActivityPhase::*;

        let expected_phase = match counter.0 {
            3 => Some(Phase1),
            6 => Some(Phase2),
            9 => Some(Phase3),
            _ => None,
        };

        if let Some(phase) = expected_phase {
            assert_eq!(trigger.event().phase, phase);
        }
    }
}

#[test]
fn test_variable_duration() {
    use {SimpleActivityPhase::*, TickState::*};

    App::new()
        .add_plugins(BaseTestPlugins)
        .add_plugins(TickPlugin)
        .init_activity::<SimpleActivity>()
        .init_resource::<TickCounter>()
        .add_systems(Startup, (spawn_active_entity, start_ticking).chain())
        .add_systems(
            FixedUpdate,
            (
                process_simple_controllers.run_if(in_state(PreTick)),
                post_tick.run_if(in_state(PostTick)),
                end_test.run_if(|counter: Res<TickCounter>| counter.0 >= 11),
            ),
        )
        .add_systems(
            OnEnter(PreTick),
            (
                increment_tick_counter,
                speed_up_phase_2.run_if(|counter: Res<TickCounter>| counter.0 == 8),
                speed_up_phase_3.run_if(|counter: Res<TickCounter>| counter.0 == 10),
            )
                .chain(),
        )
        .add_observer(on_phase_finish)
        .run();

    fn spawn_active_entity(mut commands: Commands) {
        commands.spawn((
            SimpleController,
            StatBlock::new(&[(TEST_STAT_1, 3u32), (TEST_STAT_2, 3), (TEST_STAT_3, 3)]),
        ));
    }

    fn on_phase_finish(
        trigger: On<FinishActivityPhase<SimpleActivityPhase>>,
        counter: Res<TickCounter>,
        mut stats: Single<&mut StatBlock<u32>>,
    ) {
        match trigger.event().phase {
            Phase1 => {
                assert_eq!(counter.0, 3);
                stats.set_base(TEST_STAT_2, 10);
            }
            Phase2 => assert_eq!(counter.0, 8),
            Phase3 => assert_eq!(counter.0, 10),
        }
    }

    fn speed_up_phase_2(mut stats: Single<&mut StatBlock<u32>>) {
        stats.set_base(TEST_STAT_2, 5);
    }

    fn speed_up_phase_3(mut stats: Single<&mut StatBlock<u32>>) {
        stats.set_base(TEST_STAT_3, 1);
    }
}
