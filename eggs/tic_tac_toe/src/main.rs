mod grid;
mod index;

use bevy::prelude::*;
use yanor_core::tick::start_ticking;

use grid::{GridPosition, SparseGridIndexPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SparseGridIndexPlugin::default())
        .add_systems(Startup, load_textures.before((spawn_board, spawn_players)))
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
}

fn load_textures(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
    });
}

#[derive(Component)]
struct BoardBlock;

fn spawn_board(mut commands: Commands) {
    for x in 0..3 {
        for z in 0..3 {
            commands.spawn((
                BoardBlock,
                GridPosition(IVec3::new(x, 0, z)),
                // todo!()
            ));
        }
    }
}

#[derive(Component)]
enum Player {
    X,
    O,
}

fn spawn_players(mut commands: Commands) {
    commands.spawn((
        Player::X,
        // todo
    ));

    commands.spawn((
        Player::O,
        // todo
    ));
}

