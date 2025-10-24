pub mod camera;

use bevy::prelude::*;
use yanor_core::{
    activity::FinishActivityPhase,
    animate_tick::{AnimationQueue, PendingAnimation},
    grid::GridPosition,
};

pub struct PresentationPlugin;

use crate::{
    Block, Player, Potion,
    items::{Inventory, Item, StoredIn},
    presentation::camera::*,
    step::StepPhase,
};

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_asset_handles)
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_track_player, update_billboard_transforms))
            .add_observer(on_add_block)
            .add_observer(on_add_player)
            .add_observer(on_add_potion)
            .add_observer(queue_step_animation_observer)
            .add_observer(reposition_unstored_item);
    }
}

#[derive(Resource)]
pub struct AssetHandles {
    block_mesh_handle: Handle<Mesh>,
    rect_mesh_handle: Handle<Mesh>,
    block_material_handle: Handle<StandardMaterial>,
    highlight_material_handle: Handle<StandardMaterial>,
    player_material_handle: Handle<StandardMaterial>,
    potion_material_handle: Handle<StandardMaterial>,
}

fn init_asset_handles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AssetHandles {
        block_mesh_handle: meshes.add(Cuboid::default()),
        rect_mesh_handle: meshes.add(Rectangle::default()),
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
    pos_query: Query<&GridPosition, With<Block>>,
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
    pos_query: Query<&GridPosition, With<Player>>,
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
    pos_query: Query<&GridPosition, With<Potion>>,
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
fn queue_step_animation_observer(
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

fn reposition_unstored_item(
    trigger: On<Remove, StoredIn>,
    stored_in_query: Query<&StoredIn>,
    inventory_pos_query: Query<&GridPosition, With<Inventory>>,
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
