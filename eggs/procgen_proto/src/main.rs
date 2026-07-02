mod blocks;
mod islands;
mod noiz_ext;
mod presentation;

// TODO: find non std-based alternative for generating rng seeds
use std::time::SystemTime;

use bevy::{
    camera::primitives::Aabb,
    camera_controller::free_camera::FreeCamera,
    color::palettes::css::RED,
    gizmos::GizmoPlugin,
    image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor},
    light::SunDisk,
    prelude::*,
};

use crate::{
    blocks::Block,
    islands::{
        ChunkedIslandGenerator, ChunkedIslandPlugin, IslandGenerationState, IslandGenerator,
        IslandPlugin, debug::PathPlacementDebugViz,
    },
    presentation::PresentationPlugin,
};

fn main() {
    App::new()
        // TODO: only add minimal plugins needed for game logic here, add rendering-related ones
        // in PresentationPlugin
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Nearest,
                ..default()
            },
        }))
        .add_plugins((ChunkedIslandPlugin, IslandPlugin, PresentationPlugin))
        .init_resource::<BlockHandles>()
        .add_systems(PreStartup, init)
        // .add_systems(Update, island_bounds_debug_viz)
        .run();
}

fn init(
    mut commands: Commands,
    mut next_island_gen_state: ResMut<NextState<IslandGenerationState>>,
) {
    let inland_dimensions = uvec3(60, 30, 60);
    let boundary_size = 10;
    let full_dimensions = inland_dimensions + uvec3(2 * boundary_size, 0, 2 * boundary_size);
    let island_scale = full_dimensions.as_vec3();
    let island_origin = IVec3::ZERO;
    let island_center = island_origin.as_vec3() + 0.5 * island_scale;

    commands.spawn((
        Camera3d::default(),
        FreeCamera::default(),
        // Atmosphere::EARTH,
        // Transform::from_translation(island_center + vec3(0.0, 25.0, -20.0))
        //     .looking_at(island_center, Dir3::Y),
        Transform::from_translation(vec3(2.0, 2.0, 2.0)).looking_at(Vec3::ZERO, Dir3::Y),
        AmbientLight {
            // color: todo!(),
            brightness: 500.0,
            // affects_lightmapped_meshes: todo!(),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            // color: todo!(),
            illuminance: 24000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        SunDisk::EARTH,
        Transform::from_translation(vec3(0.0, 60.0, -60.0)).looking_at(island_center, Dir3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            // color: todo!(),
            illuminance: 1500.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(island_center + vec3(0.0, 10.0, 0.0))
            .looking_at(island_center, Dir3::Y),
    ));

    // let seed = SystemTime::now()
    //     .duration_since(SystemTime::UNIX_EPOCH)
    //     .expect("time enough at last") // TODO
    //     .as_secs() as u32;

    let seed = 1781980629;

    info!("spawning island gen, seed: {seed}");

    // commands.spawn(IslandGenerator::new(seed, inland_dimensions, boundary_size));

    commands.spawn((
        Name::new("Island generator"),
        ChunkedIslandGenerator {
            seed,
            island_size: uvec2(5, 5),
        },
    ));

    // commands.spawn(PathPlacementDebugViz);

    next_island_gen_state.set(IslandGenerationState::SpawnChunkGenerators);
}

pub trait HandleProvider<A: Asset, C: Component> {
    fn handle(self, component: &C) -> Handle<A>;
}

#[derive(Resource, Clone)]
pub struct BlockHandles {
    block_mesh: Handle<Mesh>,
    dirt_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    stone_material: Handle<StandardMaterial>,
}

impl FromWorld for BlockHandles {
    fn from_world(world: &mut World) -> Self {
        let block_mesh = {
            let mut meshes = world
                .get_resource_mut::<Assets<Mesh>>()
                .expect("Assets<Mesh> resource should exist (check plugins)");

            meshes.add(Cuboid::from_length(1.0))
        };

        let dirt_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("Assets<StandardMaterial> resource should exist (check plugins)");

            materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.15, 0.1),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                // unlit: true,
                ..default()
            })
        };

        let grass_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("Assets<StandardMaterial> resource should exist (check plugins)");

            materials.add(StandardMaterial {
                base_color: Color::srgb(0.15, 0.3, 0.1),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                // unlit: true,
                ..default()
            })
        };

        let path_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("Assets<StandardMaterial> resource should exist (check plugins)");

            materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.2, 0.1),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                // unlit: true,
                ..default()
            })
        };

        let stone_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("Assets<StandardMaterial> resource should exist (check plugins)");

            materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.2),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                // unlit: true,
                ..default()
            })
        };

        BlockHandles {
            block_mesh,
            dirt_material,
            grass_material,
            path_material,
            stone_material,
        }
    }
}

impl<B: Block> HandleProvider<Mesh, B> for BlockHandles {
    fn handle(self, _component: &B) -> Handle<Mesh> {
        self.block_mesh.clone()
    }
}

impl<A: Asset, C: Component, T> HandleProvider<A, C> for Res<'_, T>
where
    T: HandleProvider<A, C> + Resource + Clone,
{
    fn handle(self, component: &C) -> Handle<A> {
        self.into_inner().clone().handle(component)
    }
}
