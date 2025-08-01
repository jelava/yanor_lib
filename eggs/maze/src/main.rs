mod player;

use bevy::prelude::*;
use yanor_core::{grid::*, input::*, tick::*};

use crate::player::Player;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins((
            InputControllerPlugin,
            SparseGridIndexPlugin::default(),
            TickPlugin,
        ))
        .add_systems(Startup, (init_asset_handles, spawn_stuff).chain())
        .add_systems(PostStartup, start_ticking)
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

const MAZE_SIZE: i32 = 25;

fn spawn_stuff(mut commands: Commands, asset_handles: Res<AssetHandles>) {
    commands.spawn((
        Player,
        InputController { queue_priority: 0 },
        GridPosition::new(0, 1, 0),
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(0.0, 5.0, -5.0), Dir3::Y),
        Mesh3d(asset_handles.player_mesh_handle.clone()),
        MeshMaterial3d(asset_handles.player_material_handle.clone()),
        Pickable {
            should_block_lower: false,
            is_hoverable: false,
        },
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, -5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Dir3::Y),
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
