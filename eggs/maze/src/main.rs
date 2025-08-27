mod animation;
mod player;
mod step;
mod ui;

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_rand::prelude::*;
use rand::Rng;
use yanor_core::{activity::*, grid::*, input::*, stats::{StatBlock, StatId}, tick::*};

use crate::{animation::*, player::*, step::*, ui::*};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins((
            EntropyPlugin::<WyRand>::default(),
            InputControllerPlugin,
            SparseGridIndexPlugin::default(),
            TickPlugin,
        ))
        .add_plugins((AnimationPresenterPlugin, PlayerPlugin, StepPlugin, UiPlugin))
        .add_systems(Startup, (init_asset_handles, spawn_stuff).chain())
        .add_systems(PostStartup, start_ticking)
        .add_systems(FixedUpdate, process_random_step_controllers.run_if(in_state(TickState::PreTick)))
        .add_systems(OnEnter(TickState::PreTick), count_ticks)
        .add_systems(Update, log_transitions::<TickState>)
        .run();
}

#[derive(Resource)]
struct AssetHandles {
    block_mesh_handle: Handle<Mesh>,
    player_mesh_handle: Handle<Mesh>,
    block_material_handle: Handle<StandardMaterial>,
    highlight_material_handle: Handle<StandardMaterial>,
    player_material_handle: Handle<StandardMaterial>,
}

fn init_asset_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AssetHandles {
        block_mesh_handle: meshes.add(Cuboid::default()),
        player_mesh_handle: meshes.add(Rectangle::default()),
        block_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("block.png")),
            unlit: true,
            ..default()
        }),
        highlight_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("highlight.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        player_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("gobbo.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
    });
}

#[derive(Component)]
struct Block;

#[derive(Component)]
struct CellHighlight;

const MAZE_SIZE: i32 = 24;

fn spawn_stuff(mut commands: Commands, asset_handles: Res<AssetHandles>) {
    let player_pos = Vec3::new(12.0, 1.0, 12.0);
    let camera_offset = Vec3::new(0.0, 8.0, -8.0);
    let camera_pos = player_pos + camera_offset;

    commands.spawn((
        Player,
        StatBlock::new(&[(MOVE_SPEED_STAT_ID, 4u32)]),
        InputController { queue_priority: 0 },
        GridPosition::new(
            player_pos.x as i32,
            player_pos.y as i32,
            player_pos.z as i32,
        ),
        Transform::from_translation(player_pos).looking_to(-camera_offset.normalize(), Dir3::Y),
        Mesh3d(asset_handles.player_mesh_handle.clone()),
        MeshMaterial3d(asset_handles.player_material_handle.clone()),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ));

    let npc_info = [
        (Vec3::new(8.0, 1.0, 12.0), 2u32),
        (Vec3::new(16.0, 1.0, 12.0), 1u32),
    ];

    for (pos, speed) in npc_info {
        commands.spawn((
            RandomStepController,
            StatBlock::new(&[(MOVE_SPEED_STAT_ID, speed)]),
            GridPosition::new(pos.x as i32, pos.y as i32, pos.z as i32),
            Transform::from_translation(pos).looking_to(-camera_offset.normalize(), Dir3::Y),
            Mesh3d(asset_handles.player_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.player_material_handle.clone()),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ));
    }

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_pos).looking_at(player_pos, Dir3::Y),
    ));

    for x in 0..MAZE_SIZE {
        for z in 0..MAZE_SIZE {
            commands
                .spawn((
                    Block,
                    GridPosition(IVec3::new(x, 0, z)),
                    Transform::from_xyz(x as f32, 0.0, z as f32),
                    Mesh3d(asset_handles.block_mesh_handle.clone()),
                    MeshMaterial3d(asset_handles.block_material_handle.clone()),
                ))
                .observe(on_block_hover);
        }
    }

    commands.spawn((
        CellHighlight,
        Transform::from_xyz(0.0, 1.0, 0.0),
        Mesh3d(asset_handles.block_mesh_handle.clone()),
        MeshMaterial3d(asset_handles.highlight_material_handle.clone()),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ));
}

fn count_ticks(mut stopwatch: Local<TickStopwatch>) {
    info!("=== tick {} ===", stopwatch.elapsed_ticks());
    stopwatch.tick(1);
}

fn on_block_hover(
    trigger: Trigger<Pointer<Over>>,
    mut cell_highlight_transform: Single<&mut Transform, With<CellHighlight>>,
    transform_query: Query<&Transform, (With<Block>, Without<CellHighlight>)>,
) {
    if let Ok(&board_transform) = transform_query.get(trigger.target()) {
        cell_highlight_transform.translation = board_transform.translation + Vec3::Y;
    } else {
        warn!("Hovered Block has no Transform");
    }
}

#[derive(Component)]
#[require(Inactive)]
struct RandomStepController;

fn process_random_step_controllers(
    mut commands: Commands,
    mut rng: GlobalEntropy<WyRand>,
    controller_query: Query<Entity, (With<RandomStepController>, With<Inactive>, With<PendingPreTick>)>,
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
