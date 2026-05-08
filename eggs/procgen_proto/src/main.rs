mod blocks;
mod islands;
mod noiz_ext;
mod presentation;

// TODO: try not to depend on std
use std::time::SystemTime;

use bevy::{dev_tools::fps_overlay::FpsOverlayPlugin, light::SunDisk, pbr::Atmosphere, prelude::*};

use crate::{
    blocks::{Block, BlocksPlugin, DirtBlock, GrassBlock, PathBlock, StoneBlock},
    islands::{block_gen::*, path_gen::*, region_graph::*, *},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FpsOverlayPlugin::default(),
        ))
        .add_plugins(BlocksPlugin)
        .init_resource::<BasicHandles>()
        .add_systems(PreStartup, init)
        .add_systems(
            Startup,
            (
                generate_first_pass_region_graph,
                generate_second_pass_region_graph,
                generate_path_edges,
                generate_path_placements,
                generate_surface_blocks,
            )
                .chain(),
        )
        // .add_systems(PostStartup, init_path_placement_debug_viz)
        .add_systems(Update, (island_bounds_debug_viz, rotate_camera))
        .run();
}

fn init(mut commands: Commands) {
    let inland_dimensions = uvec3(100, 50, 64);
    let boundary_size = 30;
    let full_dimensions = inland_dimensions + uvec3(2 * boundary_size, 0, 2 * boundary_size);
    let island_scale = full_dimensions.as_vec3();
    let island_origin = IVec3::ZERO;
    let island_center = island_origin.as_vec3() + 0.5 * island_scale;

    commands.spawn((
        Camera3d::default(),
        Atmosphere::EARTH,
        Transform::from_translation(island_center + vec3(-100.0, 20.0, -100.0))
            .looking_at(island_center, Dir3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        SunDisk::EARTH,
        Transform::from_translation(Vec3::ZERO)
            .looking_to(vec3(-0.25, -0.75, 0.25).normalize(), Dir3::Y),
    ));

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("time enough at last") // TODO
        .as_secs() as u32;

    info!("spawning island gen");

    commands.spawn((IslandGenerator::new(
        seed,
        island_origin,
        inland_dimensions,
        boundary_size,
    ),));

    // commands.spawn((
    //     IslandBoundsDebugViz,
    //     ThirdPassRegionDebugViz,
    //     PathPlacementDebugViz,
    // ));
}

fn rotate_camera(
    mut camera_transform: Single<&mut Transform, With<Camera3d>>,
    island_gen: Single<&IslandGenerator>,
) {
    camera_transform.rotate_around(island_gen.center(), Quat::from_rotation_y(0.01));
}

pub trait HandleProvider<A: Asset, C: Component> {
    fn handle(self, component: &C) -> Handle<A>;
}

#[derive(Resource, Clone)]
pub struct BasicHandles {
    block_mesh: Handle<Mesh>,
    dirt_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    stone_material: Handle<StandardMaterial>,
}

impl FromWorld for BasicHandles {
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

        BasicHandles {
            block_mesh,
            dirt_material,
            grass_material,
            path_material,
            stone_material,
        }
    }
}

impl<B: Block> HandleProvider<Mesh, B> for BasicHandles {
    fn handle(self, _component: &B) -> Handle<Mesh> {
        self.block_mesh.clone()
    }
}

impl HandleProvider<StandardMaterial, DirtBlock> for BasicHandles {
    fn handle(self, _component: &DirtBlock) -> Handle<StandardMaterial> {
        self.dirt_material.clone()
    }
}

impl HandleProvider<StandardMaterial, GrassBlock> for BasicHandles {
    fn handle(self, _component: &GrassBlock) -> Handle<StandardMaterial> {
        self.grass_material.clone()
    }
}

impl HandleProvider<StandardMaterial, PathBlock> for BasicHandles {
    fn handle(self, _component: &PathBlock) -> Handle<StandardMaterial> {
        self.path_material.clone()
    }
}

impl HandleProvider<StandardMaterial, StoneBlock> for BasicHandles {
    fn handle(self, _component: &StoneBlock) -> Handle<StandardMaterial> {
        self.stone_material.clone()
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

/*
fn weird_idea(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let chunk_size = 100;
    let image_size = uvec2(chunk_size * chunk_size, 3 * chunk_size);
    let mut data = Vec::with_capacity((4 * image_size.x * image_size.y) as usize);

    let mut plane_count = 0;

    let image = images.add(Image::new(
        Extent3d {
            width: image_size.x,
            height: image_size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::all(),
    ));

    let material = materials.add(StandardMaterial {
        // base_color_texture: Some(image),
        // unlit: true,
        cull_mode: None,
        alpha_mode: AlphaMode::Mask(1.0),
        ..default()
    });

    for x in 0..(chunk_size + 1) {
        for y in 0..chunk_size {
            for z in 0..chunk_size {
                let test = x % 2;
                let value = if (y % 2 == test) && (z % 2 == test) { 255 } else { 0 };

                data.push(value);
                data.push(value);
                data.push(value);
                data.push(value);
            }
        }


        let plane = meshes.add(Plane3d::new(Vec3::X, Vec2::splat(0.5)));
        plane_count += 1;

        commands.spawn((
            Transform::from_xyz(x as f32 / chunk_size as f32, 0.5, 0.5),
            Mesh3d(plane),
            MeshMaterial3d(material.clone()),
        ));
    }

    // for y in 0..(chunk_size + 1) {
    //     let mut data = Vec::with_capacity((4 * chunk_size * chunk_size) as usize);

    //     for x in 0..chunk_size {
    //         for z in 0..chunk_size {
    //             let test = y % 2;
    //             let value = if (x % 2 == test) && (z % 2 == test) { 255 } else { 0 };

    //             data.push(value);
    //             data.push(value);
    //             data.push(value);
    //             data.push(value);
    //         }
    //     }

    //     let image = images.add(Image::new(
    //         Extent3d {
    //             width: image_size.x,
    //             height: image_size.y,
    //             depth_or_array_layers: 1,
    //         },
    //         TextureDimension::D2,
    //         data,
    //         TextureFormat::Rgba8Unorm,
    //         RenderAssetUsages::all(),
    //     ));

    //     let plane = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(0.5 * chunk_size as f32)));
    //     plane_count += 1;

    //     let material = materials.add(StandardMaterial {
    //         // base_color_texture: Some(image),
    //         // unlit: true,
    //         cull_mode: None,
    //         alpha_mode: AlphaMode::Mask(1.0),
    //         ..default()
    //     });

    //     commands.spawn((
    //         Transform::from_xyz(0.5 * chunk_size as f32, y as f32, 0.5 * chunk_size as f32),
    //         Mesh3d(plane),
    //         MeshMaterial3d(material),
    //     ));
    // }

    // for z in 0..(chunk_size + 1) {
    //     let mut data = Vec::with_capacity((4 * chunk_size * chunk_size) as usize);

    //     for x in 0..chunk_size {
    //         for y in 0..chunk_size {
    //             let test = z % 2;
    //             let value = if (x % 2 == test) && (y % 2 == test) { 255 } else { 0 };

    //             data.push(value);
    //             data.push(value);
    //             data.push(value);
    //             data.push(value);
    //         }
    //     }

    //     let image = images.add(Image::new(
    //         Extent3d {
    //             width: image_size.x,
    //             height: image_size.y,
    //             depth_or_array_layers: 1,
    //         },
    //         TextureDimension::D2,
    //         data,
    //         TextureFormat::Rgba8Unorm,
    //         RenderAssetUsages::all(),
    //     ));

    //     let plane = meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(0.5 * chunk_size as f32)));
    //     plane_count += 1;

    //     let material = materials.add(StandardMaterial {
    //         // base_color_texture: Some(image),
    //         // unlit: true,
    //         cull_mode: None,
    //         alpha_mode: AlphaMode::Mask(1.0),
    //         ..default()
    //     });

    //     commands.spawn((
    //         Transform::from_xyz(0.5 * chunk_size as f32, 0.5 * chunk_size as f32, z as f32),
    //         Mesh3d(plane),
    //         MeshMaterial3d(material),
    //     ));
    // }

    info!("{plane_count}");
}
*/
