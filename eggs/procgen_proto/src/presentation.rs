/// For components, systems, etc. related to rendering, to keep them from being directly coupled w/ simulatiton
mod blocks;
pub mod islands;
pub mod materials;

use bevy::{
    asset::RenderAssetUsages,
    camera_controller::free_camera::FreeCameraPlugin,
    dev_tools::fps_overlay::FpsOverlayPlugin,
    ecs::{
        query::{QueryData, ROQueryItem},
        system::{SystemParam, SystemParamItem},
    },
    pbr::{
        ExtendedMaterial,
        wireframe::{Wireframe, WireframePlugin},
    },
    prelude::*,
};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::{
    HandleProvider,
    islands::chunk_gen::{ISLAND_CHUNK_SIZE, IslandChunk},
    presentation::{blocks::BlockPresentationPlugin, materials::*},
};

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                FpsOverlayPlugin::default(),
                FreeCameraPlugin,
                MaterialPlugin::<ExtendedMaterial::<StandardMaterial, PixelArtMaterialExtension>>::default(),
                WireframePlugin::default(),
            ))
            .add_plugins((
                EguiPlugin::default(),
                WorldInspectorPlugin::new(),
            ))
            .add_plugins(BlockPresentationPlugin)
            .add_observer(on_add_island_chunk);
    }
}

pub trait IntoPresentableBundle: Component + Sized {
    type MeshProvider: HandleProvider<Mesh, Self> + SystemParam;
    type MaterialProvider: HandleProvider<StandardMaterial, Self> + SystemParam;
    type Data: QueryData;

    fn into_presentable_bundle(
        &self,
        mesh_provider: SystemParamItem<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle;

    fn on_add_presentable(
        trigger: On<Add, Self>,
        mut commands: Commands,
        mesh_provider: SystemParamItem<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>,
        query: Query<(&Self, Self::Data)>,
    ) {
        let target = trigger.event_target();

        if let Ok((presentable, data)) = query.get(target) {
            commands
                .entity(target)
                .insert(presentable.into_presentable_bundle(
                    mesh_provider,
                    material_provider,
                    data,
                ));
        } else {
            warn!("Couldn't find Presentable data in query");
        }
    }
}

pub fn on_add_island_chunk(
    trigger: On<Add, IslandChunk>,
    mut commands: Commands,
    chunk_query: Query<&IslandChunk>,
) {
    let target = trigger.event_target();

    if let Ok(chunk) = chunk_query.get(target) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for z in 0..ISLAND_CHUNK_SIZE {
            for x in 0..ISLAND_CHUNK_SIZE {
                generate_chunk_mesh_tile(chunk, x, z, &mut vertices, &mut indices);
            }
        }

        let mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        // .with_inserted_attribute(
        //     PixelArtMaterialExtension::ATTRIBUTE_TEXTURE_INDEX,
        //     texture_indices,
        // )
        .with_inserted_indices(bevy::mesh::Indices::U32(indices.clone()))
        .with_computed_normals();

        let material = StandardMaterial {
            base_color: Color::srgb(0.18, 0.35, 0.12),
            metallic: 0.0,
            perceptual_roughness: 0.99,
            reflectance: 0.01,
            // unlit: true,
            // cull_mode: None,
            ..default()
        };

        commands.spawn_scene(bsn! {
            #island_chunk_mesh
            ChildOf(target)
            Mesh3d(asset_value(mesh))
            MeshMaterial3d::<StandardMaterial>(asset_value(material))
            // Transform
            // ShowAabbGizmo
            Wireframe
        });
    } else {
        warn!("couldn't find newly added chunk?");
    }
}

fn generate_chunk_mesh_tile(
    chunk: &IslandChunk,
    x: usize,
    z: usize,
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
) {
    let terrain = chunk.terrain_map[x + z * ISLAND_CHUNK_SIZE];
    let tile_pos = uvec3(x as u32, terrain.chunk_height, z as u32);
    let inner_corner_pos = tile_pos.as_vec3() + vec3(0.125, 0.0, 0.125); // TODO: not const
    let index = vertices.len() as u32;

    vertices.push(inner_corner_pos);
    vertices.push(inner_corner_pos + vec3(0.0, 0.0, 1.0 - 0.125));
    vertices.push(inner_corner_pos + vec3(1.0 - 0.125, 0.0, 1.0 - 0.125));
    vertices.push(inner_corner_pos + vec3(1.0 - 0.125, 0.0, 0.0));

    indices.push(index);
    indices.push(index + 1);
    indices.push(index + 2);

    indices.push(index);
    indices.push(index + 2);
    indices.push(index + 3);
}
