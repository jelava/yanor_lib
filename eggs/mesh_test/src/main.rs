use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

#[derive(Component)]
pub struct Rotate;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-3.0, 2.0, 2.0)
            .looking_at(Vec3::ZERO, Vec3::Y)
    ));

    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(-1.0, 3.0, 1.0),
    ));

    // commands.spawn((
    //     SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("stairs2.glb"))),
    //     Transform::from_translation(Vec3::ZERO),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(asset_server.load("stairs.png")),
    //         // base_color: Color::BLACK,
    //         unlit: true,
    //         ..default()
    //     })),
    // ));

    let mesh_label = GltfAssetLabel::Primitive{ mesh: 0, primitive: 0 }.from_asset("block.glb");
    // let mat_label = GltfAssetLabel::Material { index: 0, is_scale_inverted: false, }.from_asset("stairs.glb");
    // let material: Handle<StandardMaterial> = asset_server.load(mat_label);

    commands.spawn((
        Rotate,
        Transform::from_translation(Vec3::ZERO),
        Mesh3d(asset_server.load(mesh_label)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("block.png")),
            unlit: true,
            ..default()
        })),
    ));
}

fn rotate(mut transform: Single<&mut Transform, With<Rotate>>) {
    transform.rotate_y(0.02);
}
