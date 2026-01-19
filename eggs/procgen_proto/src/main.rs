mod islands;
mod presentation;

use std::{cmp::{max, min}, f32::{self, consts::SQRT_2}, time::SystemTime};

use bevy::{
    asset::RenderAssetUsages, dev_tools::fps_overlay::FpsOverlayPlugin, ecs::{query::{QueryData, ROQueryItem}, system::{ReadOnlySystemParam, SystemParam, SystemParamItem}}, light::SunDisk, math::{FloatPow, VectorSpace}, pbr::Atmosphere, prelude::*, render::render_resource::{Extent3d, Face, TextureDimension, TextureFormat}
};
use noiz::{Noise, Sampleable, SampleableFor, ScalableNoise, SeedableNoise, cell_noise::{DistanceToEdge, PerNearestPoint, WorleyAverage, WorleyDifference, WorleyProduct, WorleyRatio, WorleySecondLeastDistance, WorleySmoothMin}, cells::{CellPoint, DomainCell, OrthoGrid, Partitioner, SimplexGrid, Voronoi}, curves::{CubicSMin, Lerped, Smoothstep}, lengths::{ChebyshevLength, HybridLength, LengthFunction, MinkowskiLength}, math_noise::ReverseUNorm, misc_noise::Constant, prelude::{Billow, BlendCellGradients, BlendCellValues, DistanceBlend, EuclideanLength, FractalLayers, LayeredNoise, ManhattanLength, Masked, MixCellGradients, Normed, Octave, Offset, PerCell, PerCellPointDistances, Persistence, PingPong, QuickGradients, RandomElements, RemapCurve, SNormToUNorm, Scaled, SimplecticBlend, WorleyLeastDistance, common_noise::{self, Fbm, Perlin, PerlinWithDerivative, Simplex, Worley}}, rng::{NoiseRng, Random, SNorm, UNorm}};
use yanor_core::grid::GridPos;

use crate::{islands::{IslandGenDebugViz, IslandGenerator}, presentation::IntoPresentableBundle};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FpsOverlayPlugin::default(),
        ))
        .init_resource::<BasicHandles>()
        .add_systems(Startup, (init, island_debug_viz_init, /*generate_islands*/).chain())
        .add_systems(Update, (draw_island_gen_gizmos, rotate_camera))
        .add_observer(GrassBlock::on_add_presentable)
        .run();
}

fn init(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let island_size = uvec3(100, 100, 100);
    let island_scale = island_size.as_vec3();
    let island_origin = IVec3::ZERO;
    let island_center = island_origin.as_vec3() + 0.5 * island_scale;

    commands.spawn((
        Camera3d::default(),
        // Atmosphere::EARTH,
        // Transform::from_translation(island_center + vec3(-100.0, 100.0, -100.0))
        //     .looking_at(island_center, Dir3::Y),

        Transform::from_translation(Vec3::splat(110.0))
            .looking_at(Vec3::splat(50.0), Dir3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        // SunDisk::EARTH,
        Transform::from_translation(Vec3::ZERO)
            .looking_to(vec3(-0.25, -0.75, 0.25).normalize(), Dir3::Y),
    ));

    // commands.spawn();

    commands.spawn((
        IslandGenerator {
            origin: island_origin,
            dimensions: island_size,
            min_regions: 5,
            min_sub_regions: 5,
        },
        IslandGenDebugViz,
    ));


}

#[derive(Default)]
struct IslandRegion {
    is_edge_region: bool,
    has_feature: bool,
}

fn island_debug_viz_init(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    island_gen_query: Query<&IslandGenerator, With<IslandGenDebugViz>>,
) {
    let seed = NoiseRng(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time enough at last")
            .as_secs() as u32
    );

    // let seed = NoiseRng(1768259692);

    info!("seed: {:?}", seed.0);

    for island_gen in &island_gen_query {
        let island_origin = island_gen.origin;
        let island_size = island_gen.dimensions;
        let island_scale = island_size.as_vec3();
        let island_center = island_origin.as_vec3() + (0.5 * island_scale);

        let frequency = 5.0 / island_scale.max_element();
        let length_mode = EuclideanLength;
        let mut cells = Voronoi::<false>::default_with_randomness(0.8);

        let edge_distance_fn = DistanceToEdge::<_, _, false> {
            cells,
            length_mode,
        };

        let center_distance_fn = PerCellPointDistances {
            cells,
            length_mode,
            worley_mode: WorleyLeastDistance,
        };

        let feature_weight_curve = CurveExt::reparametrize_linear(
            SmoothStepCurve, 
            Interval::new(0.1, 0.25).unwrap()
        ).unwrap();

        let feature_weight_noise = Noise {
            noise: (
                Masked(
                    (
                        center_distance_fn,
                        ReverseUNorm,
                    ),
                    edge_distance_fn,
                ),
                RemapCurve::<_, _, true>::from(feature_weight_curve),
            ),
            seed,
            frequency,
        };

        let region_urbanicity_noise = Noise {
            noise: PerNearestPoint {
                cells,
                length_mode,
                noise: Random::<UNorm, f32>::default(),
            },
            seed,
            frequency,
        };

        let big_road_noise_curve = CurveExt::reparametrize_linear(
            SmoothStepCurve, 
            Interval::new(0.8, 0.97).unwrap()
        ).unwrap();

        let big_road_noise = Noise {
            noise: (
                edge_distance_fn,
                ReverseUNorm,
                RemapCurve::<_, _, true>::from(big_road_noise_curve),
            ),
            seed,
            frequency,
        };

        cells.randomness = 0.5;

        let edge_distance_fn = DistanceToEdge::<_, _, false> {
            cells,
            length_mode,
        };

        let small_road_noise_curve = CurveExt::reparametrize_linear(
            SmoothStepCurve, 
            Interval::new(0.7, 0.92).unwrap()
        ).unwrap();

        let small_road_noise = Noise {
            noise: (
                edge_distance_fn,
                ReverseUNorm,
                RemapCurve::<_, _, true>::from(small_road_noise_curve),
            ),
            seed,
            frequency: 2.0 * frequency,
        };

        let image_size = island_size.xz();
        let mut data = Vec::with_capacity((4 * image_size.x * image_size.y) as usize);

        for x in 0..image_size.x {
            for y in 0..image_size.y {
                let offset = vec3(x as f32, island_center.y, y as f32) + Vec3::splat(0.5);
                let pos = island_origin.as_vec3() + offset;

                let mut color = UVec3::ZERO;
                let urbanicity: f32 = region_urbanicity_noise.sample(pos);
                let mut feature_weight: f32 = 0.0;
                let big_road_weight: f32 = big_road_noise.sample(pos);
                let mut small_road_weight: f32 = 0.0;

                if urbanicity > 0.5 {
                    small_road_weight = small_road_noise.sample(pos);

                    // let center_dist_sample: f32 = region_center_dist.sample(pos);
                    feature_weight = feature_weight_noise.sample(pos);
                    info!("{feature_weight}");

                    color.y = (255.0 * feature_weight).round() as u32;
                }
        
                let road_weight = big_road_weight.max(small_road_weight);
                color.x = (255.0 * road_weight).round() as u32;

                if color.x == 255 || color.y == 255 {
                    color.z = 255;
                }

                // let traversable_weight = road_weight.max(feature_weight);
                // color = UVec3::splat((255.0 * traversable_weight).round() as u32);

                data.push(color.x as u8);
                data.push(color.y as u8);
                data.push(color.z as u8);
                data.push(255);
            }
        }

        let voronoi_image = images.add(Image::new(
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

        let plane = meshes.add(Plane3d::new(Vec3::Y, 0.5 * island_scale.xz()));

        let voronoi_material = materials.add(StandardMaterial {
            base_color_texture: Some(voronoi_image),
            unlit: true,
            alpha_mode: AlphaMode::Mask(1.0),
            ..default()
        });

        commands.spawn((
            Transform::from_translation(island_center),
            Mesh3d(plane),
            MeshMaterial3d(voronoi_material),
        ));
    }
}

fn draw_island_gen_gizmos(
    mut gizmos: Gizmos,
    island_gen_query: Query<&IslandGenerator, With<IslandGenDebugViz>>,
) {
    use bevy::color::palettes::css::*;

    for island_gen in &island_gen_query {
        let island_origin = island_gen.origin;
        let island_size = island_gen.dimensions;
        let island_scale = island_size.as_vec3();
        let island_center = island_origin.as_vec3() + (0.5 * island_scale);

        gizmos.cuboid(
            Transform::from_translation(island_center)
                .with_scale(island_scale),
            GREEN,
        );
    }
}

fn generate_islands(mut commands: Commands, island_gen_query: Query<&IslandGenerator>) {
    for island_gen in &island_gen_query {
        let corner = island_gen.origin + island_gen.dimensions.as_ivec3();
        let surface_level = (2 * corner.y) / 3;

        let mut upper_noise = Noise::<Perlin>::default();
        let mut lower_noise = Noise::<Perlin>::default();

        upper_noise.set_seed(234324);
        lower_noise.set_seed(4);

        for x in island_gen.origin.x..corner.x {
            for z in island_gen.origin.z..corner.z {
                let mut dist_from_edge = min(min(x, corner.x - x), min(z, corner.z - z)) as f32;
                dist_from_edge = dist_from_edge / max(corner.x, corner.z) as f32;
                
                let sample_vec = vec2(x as f32 / corner.x as f32, z as f32 / corner.z as f32);
                let sample = (upper_noise.sample_for::<f32>(sample_vec) + 1.0) * 0.5;
                let upper_elevation = dist_from_edge * sample * (corner.z - surface_level - 1) as f32 + 1.0;
                let surface_height = min(surface_level + upper_elevation.round() as i32, corner.y);

                for y in surface_level..surface_height {
                    info!("block at ({x}, {y}, {z})");

                    commands.spawn((
                        GrassBlock,
                        GridPos::new(x, y, z),
                    ));
                }

                let sample = (lower_noise.sample_for::<f32>(sample_vec) + 1.0) * 0.5;
                let lower_elevation = dist_from_edge * sample * (surface_level - 1) as f32;
                let bottom_depth = max(surface_level - 1 - lower_elevation.round() as i32, island_gen.origin.y);

                for y in bottom_depth..surface_level {
                    info!("block at ({x}, {y}, {z})");

                    commands.spawn((
                        DirtBlock,
                        GridPos::new(x, y, z),
                    ));
                }
            }
        }
    }
}

fn rotate_camera(mut camera_transform: Single<&mut Transform, With<Camera3d>>) {
    camera_transform.rotate_around(Vec3::splat(50.0), Quat::from_rotation_y(0.01));
}

trait HandleProvider<A: Asset, C: Component> {
    fn handle(self, component: &C) -> Handle<A>;
}

#[derive(Component)]
struct GrassBlock;

#[derive(Component)]
struct DirtBlock;

#[derive(Resource, Clone)]
struct BasicHandles {
    block_mesh: Handle<Mesh>,
    dirt_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
}

impl FromWorld for BasicHandles {
    fn from_world(world: &mut World) -> Self {
        use bevy::color::palettes::css::*;

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
                base_color: Color::Srgba(SADDLE_BROWN),
                // unlit: true,
                ..default()
            })
        };

        let grass_material = {
            let mut materials = world
                .get_resource_mut::<Assets<StandardMaterial>>()
                .expect("Assets<StandardMaterial> resource should exist (check plugins)");

            materials.add(StandardMaterial {
                base_color: Color::Srgba(GREEN),
                // unlit: true,
                ..default()
            })
        };

        BasicHandles {
            block_mesh,
            dirt_material,
            grass_material,
        }
    }
}

impl HandleProvider<Mesh, GrassBlock> for BasicHandles {
    fn handle(self, _component: &GrassBlock) -> Handle<Mesh> {
        self.block_mesh.clone()
    }
}

impl HandleProvider<StandardMaterial, GrassBlock> for BasicHandles {
    fn handle(self, _component: &GrassBlock) -> Handle<StandardMaterial> {
        self.grass_material.clone()
    }
}

impl HandleProvider<Mesh, DirtBlock> for BasicHandles {
    fn handle(self, _component: &DirtBlock) -> Handle<Mesh> {
        self.block_mesh.clone()
    }
}

impl HandleProvider<StandardMaterial, DirtBlock> for BasicHandles {
    fn handle(self, _component: &DirtBlock) -> Handle<StandardMaterial> {
        self.dirt_material.clone()
    }
}

impl<A: Asset, C: Component, T> HandleProvider<A, C> for Res<'_, T>
where T: HandleProvider<A, C> + Resource + Clone {
    fn handle(self, component: &C) -> Handle<A> {
        self.into_inner().clone().handle(component)
    }
}

impl IntoPresentableBundle for GrassBlock {
    type MeshProvider = Res<'static, BasicHandles>;
    type MaterialProvider = Res<'static, BasicHandles>;
    type Data = &'static GridPos;

    fn into_presentable_bundle(
        &self,
        mesh_provider: SystemParamItem<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Transform::from_translation(grid_pos.into()),
            Mesh3d(mesh_provider.handle(self)),
            MeshMaterial3d::<StandardMaterial>(material_provider.handle(self)),
        )
    }    
}

impl IntoPresentableBundle for DirtBlock {
    type MeshProvider = Res<'static, BasicHandles>;
    type MaterialProvider = Res<'static, BasicHandles>;
    type Data = &'static GridPos;

    fn into_presentable_bundle(
        &self,
        mesh_provider: SystemParamItem<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Transform::from_translation(grid_pos.into()),
            Mesh3d(mesh_provider.handle(self)),
            MeshMaterial3d::<StandardMaterial>(material_provider.handle(self)),
        )
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

