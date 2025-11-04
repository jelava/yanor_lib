pub mod camera;

use bevy::prelude::*;
use yanor_core::{
    activity::FinishActivityPhase,
    animate_tick::{AnimationQueue, PendingAnimation},
    grid::GridPos,
};

use crate::{
    Block, Door, DoorOrientation, Goal, Player, Potion, door::*, items::*, presentation::camera::*,
    step::*,
};

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_asset_handles)
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_track_player, update_billboard_transforms))
            .add_observer(on_add_block)
            .add_observer(on_add_player)
            .add_observer(on_add_potion)
            .add_observer(on_add_goal)
            .add_observer(on_add_door)
            .add_observer(queue_step_animation)
            .add_observer(queue_door_open_animation)
            .add_observer(queue_door_close_animation)
            .add_observer(reposition_unstored_item);
    }
}

#[derive(Resource)]
pub struct AssetHandles {
    pub block_mesh_handle: Handle<Mesh>,
    pub door_mesh_handle: Handle<Mesh>,
    pub rect_mesh_handle: Handle<Mesh>,
    pub block_material_handle: Handle<StandardMaterial>,
    pub door_material_handle: Handle<StandardMaterial>,
    pub cursor_material_handle: Handle<StandardMaterial>,
    pub player_material_handle: Handle<StandardMaterial>,
    pub potion_material_handle: Handle<StandardMaterial>,
}

fn init_asset_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AssetHandles {
        block_mesh_handle: meshes.add(Cuboid::default()),
        door_mesh_handle: meshes.add(Cuboid::new(1.0, 1.0, 0.25)),
        rect_mesh_handle: meshes.add(Rectangle::default()),
        block_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("block.png")),
            unlit: true,
            ..default()
        }),
        door_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("door_front_back.png")),
            unlit: true,
            ..default()
        }),
        cursor_material_handle: materials.add(StandardMaterial {
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
        potion_material_handle: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("potion.png")),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            cull_mode: None,
            ..default()
        }),
    });
}

fn on_add_block(
    trigger: On<Add, Block>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    pos_query: Query<&GridPos, With<Block>>,
) {
    let target = trigger.event_target();

    if let Ok(&grid_pos) = pos_query.get(target) {
        commands.entity(target).insert((
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.block_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.block_material_handle.clone()),
        ));
    } else {
        warn!("Block component added to entity without GridPosition, will not be presented");
    }
}

fn on_add_player(
    trigger: On<Add, Player>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    pos_query: Query<&GridPos, With<Player>>,
) {
    let target = trigger.event_target();

    if let Ok(&grid_pos) = pos_query.get(target) {
        commands.entity(target).insert((
            Billboard,
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.rect_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.player_material_handle.clone()),
        ));
    } else {
        warn!("Player component added to entity without GridPosition, will not be presented");
    }
}

fn on_add_potion(
    trigger: On<Add, Potion>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    pos_query: Query<&GridPos, With<Potion>>,
) {
    let target = trigger.event_target();

    if let Ok(&grid_pos) = pos_query.get(target) {
        commands.entity(target).insert((
            Billboard,
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.rect_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.potion_material_handle.clone()),
        ));
    } else {
        // If the item has no GridPosition it is probably spawning into an inventory, so set up the
        // presentation components and a temporary transform but make it not visible
        commands.entity(target).insert((
            Visibility::Hidden,
            Billboard,
            Transform::from_translation(Vec3::ZERO),
            Mesh3d(asset_handles.rect_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.potion_material_handle.clone()),
        ));
    }
}

fn on_add_goal(
    trigger: On<Add, Goal>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    pos_query: Query<&GridPos, With<Goal>>,
) {
    let target = trigger.event_target();

    if let Ok(&grid_pos) = pos_query.get(target) {
        commands.entity(target).insert((
            Transform::from_translation(grid_pos.into()),
            Mesh3d(asset_handles.block_mesh_handle.clone()),
            MeshMaterial3d(asset_handles.cursor_material_handle.clone()),
        ));
    } else {
        warn!("Goal component added to entity without GridPosition, will not be presented");
    }
}

// const DOOR_OFFSET: Vec3 = Vec3::new(0.5, 0.0, 0.0);

fn on_add_door(
    trigger: On<Add, Door>,
    mut commands: Commands,
    asset_handles: Res<AssetHandles>,
    pos_query: Query<(&GridPos, &DoorOrientation, Has<Open>), With<Door>>,
) {
    let target = trigger.event_target();

    if let Ok((&grid_pos, door_orientation, door_open)) = pos_query.get(target) {
        let (door_dir, door_offset) = match door_orientation {
            DoorOrientation::FacingZ => (Dir3::Z, 0.5 * Vec3::X),
            DoorOrientation::FacingX => (Dir3::X, 0.5 * Vec3::Z),
        };

        let door_angle = match door_open {
            true => 1.5,
            false => 0.0,
        };

        // warn!("TODO: change initial rotation of door based on whether it's open/closed");

        let child = commands
            .spawn((
                Transform::from_translation(door_offset).looking_to(door_dir, Dir3::Y),
                Mesh3d(asset_handles.door_mesh_handle.clone()),
                MeshMaterial3d(asset_handles.door_material_handle.clone()),
            ))
            .id();

        let pos: Vec3 = grid_pos.into();

        commands
            .entity(target)
            .insert((
                Transform::from_translation(pos - door_offset)
                    .with_rotation(Quat::from_rotation_y(door_angle)),
                // Mesh3d(asset_handles.door_mesh_handle.clone()),
                // MeshMaterial3d(asset_handles.door_material_handle.clone()),
            ))
            .add_child(child);
    } else {
        warn!("Goal component added to entity without GridPosition, will not be presented");
    }
}

// TODO: generalized approach to reduce boilerplate
// fn on_add_presentable<C: Component>(
//     trigger: On<Add, C>,
//     mut commands: Commands,
//     asset_handles: Res<AssetHandles>,
//     pos_query: Query<&GridPosition, With<C>>,
// ) {
//     let target = trigger.event_target();

//     if let Ok(&grid_pos) = pos_query.get(target) {
//         commands.entity(target).insert((
//             Billboard,
//             Transform::from_translation(grid_pos.into()),
//             Mesh3d(todo!()),
//             MeshMaterial3d(todo!()),
//         ));
//     } else {
//         warn!("Can't find GridPosition of presentable");
//     }
// }

// TODO: on remove/despawn handlers

// fn on_block_hover(
//     trigger: Trigger<Pointer<Over>>,
//     mut cell_highlight_transform: Single<&mut Transform, With<CellHighlight>>,
//     transform_query: Query<&Transform, (With<Block>, Without<CellHighlight>)>,
// ) {
//     if let Ok(&board_transform) = transform_query.get(trigger.target()) {
//         cell_highlight_transform.translation = board_transform.translation + Vec3::Y;
//     } else {
//         warn!("Hovered Block has no Transform");
//     }
// }

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
