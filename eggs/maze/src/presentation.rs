pub mod camera;

use bevy::{
    ecs::query::{QueryData, ROQueryItem},
    prelude::*,
};
use yanor_core::{
    activity::FinishActivityPhase,
    animate_tick::{AnimationQueue, PendingAnimation},
    grid::GridPos,
};

use crate::{
    Block, Door, Goal, Player, Potion, Stairs, XzPlaneOrientation, door::*, items::*,
    presentation::camera::*, step::*,
};

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_asset_handles)
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_track_player, update_billboard_transforms))
            .add_observer(Block::on_add_presentable)
            .add_observer(Door::on_add_presentable)
            .add_observer(Goal::on_add_presentable)
            .add_observer(Player::on_add_presentable)
            .add_observer(Potion::on_add_presentable)
            .add_observer(Stairs::on_add_presentable)
            .add_observer(queue_step_animation)
            .add_observer(queue_door_open_animation)
            .add_observer(queue_door_close_animation)
            .add_observer(reposition_unstored_item);
    }
}

// TODO: this is a rather messy and simplistic way to handle handles
#[derive(Resource)]
pub struct AssetHandles {
    pub block_mesh_handle: Handle<Mesh>,
    pub cube_mesh_handle: Handle<Mesh>,
    pub door_mesh_handle: Handle<Mesh>,
    pub rect_mesh_handle: Handle<Mesh>,
    pub stair_mesh_handle: Handle<Mesh>,
    pub block_material_handle: Handle<StandardMaterial>,
    pub door_material_handle: Handle<StandardMaterial>,
    pub cursor_material_handle: Handle<StandardMaterial>,
    pub player_material_handle: Handle<StandardMaterial>,
    pub potion_material_handle: Handle<StandardMaterial>,
    pub stair_material_handle: Handle<StandardMaterial>,
}

fn init_asset_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AssetHandles {
        block_mesh_handle: asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("meshes/block.glb"),
        ),
        cube_mesh_handle: meshes.add(Cuboid::from_length(1.0)),
        door_mesh_handle: asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("meshes/door.glb"),
        ),
        rect_mesh_handle: meshes.add(Rectangle::default()),
        stair_mesh_handle: asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("meshes/stairs.glb"),
        ),
        block_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/block.png")),
            unlit: true,
            ..default()
        }),
        door_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/door.png")),
            unlit: true,
            ..default()
        }),
        cursor_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/highlight.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        player_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/gobbo.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        potion_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/potion.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
        stair_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("textures/stairs.png")),
            unlit: true,
            ..default()
        }),
    });
}

trait IntoPresentableBundle: Component + Sized {
    type Data: QueryData;
    // type Filter: QueryFilter;

    // TODO: don't rely directly on AssetHandles?
    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle;

    fn on_add_presentable(
        trigger: On<Add, Self>,
        mut commands: Commands,
        asset_handles: Res<AssetHandles>,
        query: Query<Self::Data, With<Self>>,
    ) {
        let target = trigger.event_target();

        if let Ok(data) = query.get(target) {
            commands
                .entity(target)
                .insert(Self::into_presentable_bundle(asset_handles, data));
        } else {
            warn!("Couldn't find Presentable data in query");
        }
    }
}

impl IntoPresentableBundle for Block {
    // static???
    type Data = &'static GridPos;
    // type Filter = With<Self>;

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.block_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.block_material_handle.clone()),
        )
    }
}

impl IntoPresentableBundle for Player {
    type Data = &'static GridPos;

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Billboard,
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.rect_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.player_material_handle.clone()),
        )
    }
}

impl IntoPresentableBundle for Potion {
    type Data = Option<&'static GridPos>;

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        if let Some(&grid_pos) = data {
            (
                Visibility::Inherited,
                Billboard,
                Transform::from_translation(grid_pos.into()),
                Mesh3d(asset_handles.rect_mesh_handle.clone()),
                MeshMaterial3d(asset_handles.potion_material_handle.clone()),
            )
        } else {
            // If the item has no GridPosition it is probably spawning into an inventory, so set up the
            // presentation components and a temporary transform but make it not visible
            (
                Visibility::Hidden,
                Billboard,
                Transform::from_translation(Vec3::ZERO),
                Mesh3d(asset_handles.rect_mesh_handle.clone()),
                MeshMaterial3d(asset_handles.potion_material_handle.clone()),
            )
        }
    }
}

impl IntoPresentableBundle for Goal {
    type Data = &'static GridPos;

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.block_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.cursor_material_handle.clone()),
        )
    }
}

impl IntoPresentableBundle for Door {
    type Data = (&'static GridPos, &'static XzPlaneOrientation, Has<Open>);

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        use XzPlaneOrientation::*;

        let (&grid_pos, door_orientation, door_open) = data;

        let (door_dir, door_offset) = match door_orientation {
            FacingZ => (Dir3::Z, 0.5 * Vec3::NEG_X),
            FacingNegZ => (Dir3::Z, 0.5 * Vec3::NEG_X),
            FacingX => (Dir3::X, 0.5 * Vec3::Z),
            FacingNegX => (Dir3::X, 0.5 * Vec3::Z),
        };

        let door_angle = match door_open {
            true => 1.5,
            false => 0.0,
        };

        let pos: Vec3 = grid_pos.into();

        (
            Transform::from_translation(pos - door_offset)
                .with_rotation(Quat::from_rotation_y(door_angle)),
            Visibility::default(),
            children![(
                Transform::from_translation(door_offset).looking_to(door_dir, Dir3::Y),
                Mesh3d(asset_handles.door_mesh_handle.clone()),
                MeshMaterial3d(asset_handles.door_material_handle.clone()),
            )],
        )
    }
}

impl IntoPresentableBundle for Stairs {
    type Data = (&'static GridPos, &'static XzPlaneOrientation);

    fn into_presentable_bundle(
        asset_handles: Res<AssetHandles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let (&grid_pos, &orientation) = data;

        (
            Transform::from_translation(grid_pos.into()).looking_to(orientation, Dir3::Y),
            Mesh3d(asset_handles.stair_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.stair_material_handle.clone()),
        )
    }
}

// TODO: on remove/despawn handlers

// For sequential animation, preserving information about the order in which changes within the
// tick happened is useful, so queuing happens immediately via observer rather than waiting
// until PostTick to check for differences between the Transform and the GridPosition (for example)
fn queue_step_animation(
    trigger: On<FinishActivityPhase<StepPhase>>,
    mut commands: Commands,
    mut anim_queue: ResMut<AnimationQueue>,
) {
    if trigger.event().phase == StepPhase::BeginStep {
        let entity = trigger.event_target();
        anim_queue.push_back(entity);
        commands.entity(entity).insert(PendingAnimation::Step);
    }
}

fn queue_door_open_animation(
    trigger: On<Remove, Closed>, // Remove rather than Insert to avoid extraneous animation when door is spawned
    mut commands: Commands,
    mut anim_queue: ResMut<AnimationQueue>,
    door_query: Query<Entity, With<Door>>,
) {
    let target = trigger.event_target();

    if door_query.contains(target) {
        anim_queue.push_back(target);
        commands.entity(target).insert(PendingAnimation::OpenDoor);
    }
}

fn queue_door_close_animation(
    trigger: On<Remove, Open>,
    mut commands: Commands,
    mut anim_queue: ResMut<AnimationQueue>,
    door_query: Query<Entity, With<Door>>,
) {
    let target = trigger.event_target();

    if door_query.contains(target) {
        anim_queue.push_back(target);
        commands.entity(target).insert(PendingAnimation::CloseDoor);
    }
}

fn reposition_unstored_item(
    trigger: On<Remove, StoredIn>,
    stored_in_query: Query<&StoredIn>,
    inventory_pos_query: Query<&GridPos, With<Inventory>>,
    mut item_pos_query: Query<&mut Transform, With<Item>>,
) {
    let item_entity = trigger.event_target();

    if let Ok(&StoredIn(inventory_entity)) = stored_in_query.get(item_entity) {
        if let Ok(&inventory_grid_pos) = inventory_pos_query.get(inventory_entity) {
            if let Ok(mut transform) = item_pos_query.get_mut(item_entity) {
                transform.translation = inventory_grid_pos.into();
            } else {
                warn!("item pos info not found");
            }
        } else {
            warn!("inventory pos not found");
        }
    } else {
        warn!("related inventory not found");
    }
}
