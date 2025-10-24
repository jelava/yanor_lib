mod items;
mod player;
mod presentation;
mod step;
mod ui;

use bevy::{
    dev_tools::{fps_overlay::FpsOverlayPlugin, states::log_transitions},
    prelude::*,
    time::Stopwatch,
};
use bevy_rand::prelude::*;
use rand::Rng;
use yanor_core::{activity::*, animate_tick::*, grid::*, input::*, stats::StatBlock, tick::*};

use crate::{items::*, player::*, presentation::*, step::*, ui::*};

#[derive(Resource, Default)]
struct TickPhaseCounter {
    total: u32,
    pre_tick: u32,
    tick: u32,
    post_tick: u32,
}

#[derive(Resource, Default)]
struct StepLatencyStopwatch(pub Stopwatch);

fn main() {
    App::new()
        .add_plugins((
            // bevy plugins
            DefaultPlugins,
            FpsOverlayPlugin::default(),
            MeshPickingPlugin,
        ))
        .add_plugins((
            // yanor lib plugins
            AnimateTickPlugin,
            EntropyPlugin::<WyRand>::default(),
            InputControllerPlugin,
            SparseGridIndexPlugin::default(),
            TickPlugin,
        ))
        .add_plugins((
            // local plugins
            ItemPlugin,
            PlayerPlugin,
            PresentationPlugin,
            StepPlugin,
            UiPlugin,
        ))
        // .insert_resource(Time::<Fixed>::from_hz(0.1 * 60.0))
        .init_resource::<TickPhaseCounter>()
        .init_resource::<StepLatencyStopwatch>()
        .add_systems(Startup, spawn_stuff)
        .add_systems(PostStartup, start_ticking)
        .add_systems(
            FixedUpdate,
            (
                process_random_step_controllers.run_if(in_state(TickState::PreTick)),
                update_tick_phase_counter,
            ),
        )
        .add_systems(OnEnter(TickState::PreTick), count_ticks)
        // .add_systems(Update, log_transitions::<TickState>)
        .add_systems(Update, update_step_latency_stopwatch)
        .run();

    fn update_tick_phase_counter(
        tick_state: Res<State<TickState>>,
        mut counter: ResMut<TickPhaseCounter>,
    ) {
        counter.total += 1;

        match tick_state.get() {
            TickState::PreTick => {
                counter.pre_tick += 1;
            }
            TickState::Tick => {
                counter.tick += 1;
            }
            TickState::PostTick => {
                counter.post_tick += 1;
            }
            _ => {}
        };
    }

    fn update_step_latency_stopwatch(
        time: Res<Time<Real>>,
        mut stopwatch: ResMut<StepLatencyStopwatch>,
    ) {
        stopwatch.0.tick(time.delta());
    }
}

#[derive(Component)]
struct Block;

#[derive(Component)]
#[require(Item)]
struct Potion;

const MAZE_SIZE: i32 = 24;

fn spawn_stuff(mut commands: Commands) {
    let player_pos = Vec3::new(12.0, 1.0, 12.0);
    let camera_offset = Vec3::new(0.0, 8.0, -8.0);
    let camera_pos = player_pos + camera_offset;

    commands.spawn((
        Player,
        InputController { queue_priority: 0 },
        // RandomStepController,
        GridPosition::new(
            player_pos.x as i32,
            player_pos.y as i32,
            player_pos.z as i32,
        ),
        StatBlock::new(&[
            (MOVE_DURATION_STAT_ID, 1u32),
            (GRAB_ITEM_DURATION_STAT_ID, 1u32),
            (STORE_ITEM_DURATION_STAT_ID, 1u32),
            (TAKE_OUT_ITEM_DURATION_STAT_ID, 1u32),
            (PLACE_ITEM_DURATION_STAT_ID, 1u32),
        ]),
        Inventory::default(),
    ));

    commands.spawn((Potion, GridPosition::new(14, 1, 14)));

    for x in 0..MAZE_SIZE {
        for z in 0..MAZE_SIZE {
            commands.spawn((Block, GridPosition(IVec3::new(x, 0, z))));
            // .observe(on_block_hover);
        }
    }
}

fn count_ticks(mut stopwatch: Local<TickStopwatch>, mut counter: ResMut<TickPhaseCounter>) {
    // info!("=== tick {} ===", stopwatch.elapsed_ticks());
    stopwatch.tick(1);

    // info!(
    //     // "pre-tick: {} ({}%)\ntick: {} ({}%)\npost-tick: {} ({}%)\ntotal: {} (fupds/tick)",
    //     // counter.pre_tick,
    //     // (counter.pre_tick as f32) / (counter.total as f32) * 100.0,
    //     // counter.tick,
    //     // (counter.tick as f32) / (counter.total as f32) * 100.0,
    //     // counter.post_tick,
    //     // (counter.post_tick as f32) / (counter.total as f32) * 100.0,
    //     "{} (fupds/tick)",
    //     counter.total,
    // );

    counter.total = 0;
    counter.pre_tick = 0;
    counter.tick = 0;
    counter.post_tick = 0;
}

#[derive(Component)]
#[require(Inactive)]
struct RandomStepController;

fn process_random_step_controllers(
    mut commands: Commands,
    mut rng: Single<&mut WyRand, With<GlobalRng>>,
    controller_query: Query<
        Entity,
        (
            With<RandomStepController>,
            With<Inactive>,
            With<PendingPreTick>,
        ),
    >,
) {
    use AxisDirection::*;

    for entity in controller_query {
        let x_dir = match rng.random_range(0..3) {
            0 => Zero,
            1 => Plus,
            _ => Minus,
        };

        let z_dir = match rng.random_range(0..3) {
            0 => Zero,
            1 => Plus,
            _ => Minus,
        };

        commands
            .entity(entity)
            .insert(Active(Step(GridDirection::new(x_dir, Zero, z_dir))));
    }
}
