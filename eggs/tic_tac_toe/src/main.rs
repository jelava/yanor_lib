mod grid;
mod index;

use bevy::prelude::*;
use yanor_core::tick::{TickPlugin, start_ticking};

use grid::{GridPosition, SparseGridIndexPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins((SparseGridIndexPlugin::default(), TickPlugin))
        .add_systems(Startup, (load_textures, spawn_game).chain())
        .add_systems(PostStartup, start_ticking)
        .run();
}

#[derive(Resource)]
struct AssetHandles {
    block_mesh_handle: Handle<Mesh>,
    rect_mesh_handle: Handle<Mesh>,
    block_material_handle: Handle<StandardMaterial>,
    x_material_handle: Handle<StandardMaterial>,
    o_material_handle: Handle<StandardMaterial>,
    highlight_material_handle: Handle<StandardMaterial>,
}

fn load_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("loading textures...");

    commands.insert_resource(AssetHandles {
        block_mesh_handle: meshes.add(Cuboid::default()),
        rect_mesh_handle: meshes.add(Rectangle::default()),
        block_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("board.png")),
            unlit: true,
            ..default()
        }),
        x_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("x.png")),
            unlit: true,
            ..default()
        }),
        o_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("o.png")),
            unlit: true,
            ..default()
        }),
        highlight_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("highlight.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            ..default()
        }),
    });
}

#[derive(Component)]
enum Player {
    X,
    O,
}

#[derive(Component)]
struct BoardBlock;

#[derive(Component)]
struct CellHighlight;

fn spawn_game(mut commands: Commands, asset_handles: Res<AssetHandles>) {
    info!("spawning stuff...");

    for x in 0..3 {
        for z in 0..3 {
            commands
                .spawn((
                    BoardBlock,
                    GridPosition(IVec3::new(x, 0, z)),
                    Transform::from_xyz(x as f32, 0.0, z as f32),
                    Mesh3d(asset_handles.block_mesh_handle.clone()),
                    MeshMaterial3d(asset_handles.block_material_handle.clone()),
                ))
                .observe(on_board_hover);
        }
    }

    commands.spawn((
        CellHighlight,
        Transform::from_xyz(0.0, 1.0, 0.0),
        Mesh3d(asset_handles.block_mesh_handle.clone()),
        MeshMaterial3d(asset_handles.highlight_material_handle.clone()),
    ));

    commands.spawn((
        Player::X,
        // todo
    ));

    commands.spawn((
        Player::O,
        // todo
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.0, 4.0, -2.0).looking_at(Vec3::new(1.0, 0.0, 1.0), Dir3::Y),
    ));
}

fn on_board_hover(
    trigger: Trigger<Pointer<Over>>,
    mut cell_highlight: Single<&mut Transform, With<CellHighlight>>,
    transform_query: Query<&Transform, (With<BoardBlock>, Without<CellHighlight>)>,
) {
    if let Ok(&transform) = transform_query.get(trigger.target()) {
        cell_highlight.translation = transform.translation + Vec3::Y;
    } else {
        warn!("Hovered BoardBlock has no Transform?");
    }
}
