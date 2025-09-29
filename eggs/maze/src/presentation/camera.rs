use bevy::prelude::*;

use crate::player::Player;

// Marker component to indicate that a camera is being used for this particular presentation layer.
// Should only be added to exactly one camera.
#[derive(Component)]
#[require(Camera3d)]
pub struct PresentationCamera;

#[derive(Component)]
pub struct Billboard;

#[derive(Component, Clone, Copy)]
pub struct PlayerTracking {
    offset: Vec3,
}

pub fn spawn_camera(mut commands: Commands) {
    // TODO: don't hardcode this, load from config or something
    let offset = Vec3::new(0.0, 8.0, -8.0);

    commands.spawn((
        PresentationCamera,
        PlayerTracking { offset },
        Transform::from_translation(Vec3::ZERO).looking_at(-offset, Vec3::Y),
    ));
}

// TODO: don't use player component here, make more general component to indicate what camera
// should be tracking (or use relationship?)
pub fn camera_track_player(
    mut camera_info: Single<(&mut Transform, &PlayerTracking), With<PresentationCamera>>,
    player_transform: Single<&Transform, (With<Player>, Without<PresentationCamera>)>,
) {
    camera_info.0.translation = player_transform.translation + camera_info.1.offset;
}

pub fn update_billboard_transforms(
    camera_transform: Single<&Transform, With<PresentationCamera>>,
    mut billboards_query: Query<&mut Transform, (With<Billboard>, Without<PresentationCamera>)>,
) {
    for mut transform in &mut billboards_query {
        transform.look_to(
            camera_transform.forward().normalize() * Vec3::new(1.0, 0.0, 1.0),
            camera_transform.up().normalize(),
        );
    }
}
