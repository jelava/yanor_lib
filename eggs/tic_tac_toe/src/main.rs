use bevy::prelude::*;
use yanor_core::{
    grid::{GridPos, SparseGridIndex, SparseGridIndexPlugin},
    index::ComponentIndex,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_plugins(SparseGridIndexPlugin::default())
        .init_state::<TurnState>()
        .add_systems(Startup, (load_textures, spawn_game).chain())
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
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        o_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("o.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        highlight_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("highlight.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
    });
}

// #[derive(Component)]
// enum Player {
//     X,
//     O,
// }

#[derive(Component)]
enum Marker {
    X,
    O,
}

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
enum TurnState {
    #[default]
    XTurn,
    OTurn,
}

#[derive(Component)]
struct BoardBlock;

#[derive(Component)]
struct CellHighlight;

fn spawn_game(mut commands: Commands, asset_handles: Res<AssetHandles>) {
    for x in 0..3 {
        for z in 0..3 {
            commands
                .spawn((
                    BoardBlock,
                    GridPos(IVec3::new(x, 0, z)),
                    Transform::from_xyz(x as f32, 0.0, z as f32),
                    Mesh3d(asset_handles.block_mesh_handle.clone()),
                    MeshMaterial3d(asset_handles.block_material_handle.clone()),
                ))
                .observe(on_board_hover)
                .observe(on_board_click);
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

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.0, 4.0, -2.0).looking_at(Vec3::new(1.0, 0.0, 1.0), Dir3::Y),
    ));
}

fn on_board_hover(
    trigger: On<Pointer<Over>>,
    mut cell_highlight_transform: Single<&mut Transform, With<CellHighlight>>,
    transform_query: Query<&Transform, (With<BoardBlock>, Without<CellHighlight>)>,
) {
    if let Ok(&board_transform) = transform_query.get(trigger.target()) {
        cell_highlight_transform.translation = board_transform.translation + Vec3::Y;
    } else {
        warn!("Hovered BoardBlock has no Transform");
    }
}

fn on_board_click(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    current_turn: Res<State<TurnState>>,
    mut next_turn: ResMut<NextState<TurnState>>,
    grid_index: Res<SparseGridIndex>,
    camera_transform: Single<&Transform, With<Camera>>,
    cell_highlight_transform: Single<&Transform, With<CellHighlight>>,
) {
    let grid_pos = GridPos(IVec3::new(
        cell_highlight_transform.translation.x as i32,
        cell_highlight_transform.translation.y as i32,
        cell_highlight_transform.translation.z as i32,
    ));

    if grid_index.get(&grid_pos).is_none() {
        let (marker, material_handle, next_state) = match current_turn.get() {
            TurnState::XTurn => (
                Marker::X,
                asset_handles.x_material_handle.clone(),
                TurnState::OTurn,
            ),
            TurnState::OTurn => (
                Marker::O,
                asset_handles.o_material_handle.clone(),
                TurnState::XTurn,
            ),
        };

        commands.spawn((
            marker,
            grid_pos,
            cell_highlight_transform
                .clone()
                .looking_to(camera_transform.back(), camera_transform.up()),
            Mesh3d(asset_handles.rect_mesh_handle.clone()),
            MeshMaterial3d(material_handle),
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
        ));

        next_turn.set(next_state);
    }
}
