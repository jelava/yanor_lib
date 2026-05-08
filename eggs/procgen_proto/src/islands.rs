pub mod block_gen;
pub mod path_gen;
pub mod region_graph;

/// Procedural generation of floating islands
// General process:
//  - inputs: bounding box size, other params?
//      - create "island map" first
//          - 2d heightmap of island plus extra "layers" w/ data for later steps of gen process
//              - why 2d? fully 3d level gen adds lots of extra complications to consider (camera/visibility management alone is cumbersome for seeing what lots of units are doing at once if some are inside structures/caves)
//          - 1st pass
//              - region generation, determine which regions are inland
//                  - also calculate/store region centroids
//                  - centroid of all inland regions (avg of inland region centroids?)
//              - choose region-level properties, which regions have features, etc.
//              - if insufficient # of regions/features based on config, pick new seed and retry?
//              - also do feature placement here (or does it need its own pass?)
//                  - mandatory features
//                      - spawn/start area
//                      - star altar?
//                      - harbor? (exit)
//              - also do base/smooth elevation gen here? (part of region properties?)
//          - 2nd pass (path gen)
//              - all mandatory features need to be connected via at least 1 path
//              - can also generate optional paths to/through other features
//              - paths should roughly follow landscape (i.e. go along/around steep slopes rather than directly over)
//              -
//
//     - block generation
//         - surface generation
//             - use heightmap to determine how high to place block
//             - check other layers
//     - region details
//         - voronoi cells w/ randomly generated "id" value stored in island map layer
//         - ids are technically not guaranteed to be unique but shouldn't be a huge issue when collisions happen
//             - basically would result in a bigger, disconnected region composed of 2+ cells that would share per-region properties
//             - regions contained w/in inner "inland" bounding box form inner shape of island that's guaranteed to have terrain
//             - regions that touch/cross edge of outer bounding box are forbidden and never contain terrain
//             - any other regions are boundary/"shore" regions that may or may not contain terrain and aren't guaranteed to be traversable
//         - regions are useful for containing areas that need to be non-intersecting
//         - region properties
//             - has feature (bool): each region can potent
use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use noiz::{
    Noise, SampleableFor,
    cell_noise::PerNearestPoint,
    cells::Voronoi,
    lengths::HybridLength,
    prelude::{
        EuclideanLength, FractalLayers, LayeredNoise, Normed, Octave, PerCellPointDistances,
        Persistence, SNormToUNorm, WorleyLeastDistance,
        common_noise::{Perlin, Simplex},
    },
    rng::{NoiseRng, Random},
};
use yanor_core::grid::GridPos;

use crate::{
    blocks::{DirtBlock, GrassBlock, PathBlock, StoneBlock},
    islands::{path_gen::*, region_graph::*},
    noiz_ext::FullRange,
};

#[derive(Component)]
pub struct IslandGenerator {
    /// RNG seed
    pub seed: u32,

    /// Lower left corner of the full bounding box of the island
    pub origin: IVec3,

    /// Size of the inner bounding box containing the "inland" area that's guaranteed to have solid ground
    pub inland_dimensions: UVec3,

    /// Size of the boundary area around each edge of the inner bounding box where the "shore" of the island is
    pub boundary_size: u32,

    /// Generates (most likely) unique IDs for each region
    pub region_id_noise: Noise<PerNearestPoint<Voronoi, HybridLength, Random<FullRange, u32>>>,

    /// Used to find the cell with the lowest distance to center in the region, which is considered it's "center"
    pub region_center_distance_noise:
        Noise<PerCellPointDistances<Voronoi, HybridLength, WorleyLeastDistance>>,

    pub base_elevation_noise: Noise<(Simplex, SNormToUNorm)>,

    pub high_freq_elevation_noise: Noise<(
        LayeredNoise<Normed<f32>, Persistence, FractalLayers<Octave<Simplex>>>,
        SNormToUNorm,
    )>,
}

const DEFAULT_NOISE_FREQUENCY: f32 = 0.08;
const DEFAULT_VORONOI_RANDOMNESS: f32 = 0.8;

impl IslandGenerator {
    pub fn new(seed: u32, origin: IVec3, inland_dimensions: UVec3, boundary_size: u32) -> Self {
        let voronoi = Voronoi::<false>::default_with_randomness(DEFAULT_VORONOI_RANDOMNESS);
        let length_mode = HybridLength;
        let seed_rng = NoiseRng(seed);

        Self {
            seed,
            origin,
            inland_dimensions,
            boundary_size,
            region_id_noise: Noise {
                noise: PerNearestPoint {
                    cells: voronoi,
                    noise: Random::<FullRange, u32>::default(),
                    length_mode,
                },
                seed: seed_rng,
                frequency: DEFAULT_NOISE_FREQUENCY,
            },
            region_center_distance_noise: Noise {
                noise: PerCellPointDistances {
                    cells: voronoi,
                    length_mode,
                    worley_mode: WorleyLeastDistance,
                },
                seed: seed_rng,
                frequency: DEFAULT_NOISE_FREQUENCY,
            },
            base_elevation_noise: Noise {
                noise: (Simplex::default(), SNormToUNorm),
                seed: seed_rng,
                frequency: 0.25 * DEFAULT_NOISE_FREQUENCY,
            },
            high_freq_elevation_noise: Noise {
                noise: (
                    LayeredNoise::new(
                        Normed::<f32>::default(),
                        Persistence::default(),
                        FractalLayers {
                            layer: Octave(Simplex::default()),
                            lacunarity: 2.0,
                            amount: 8,
                        },
                    ),
                    SNormToUNorm,
                ),
                seed: seed_rng,
                frequency: 0.25 * DEFAULT_NOISE_FREQUENCY,
            },
        }
    }

    pub fn full_dimensions(&self) -> UVec3 {
        self.inland_dimensions + uvec3(2 * self.boundary_size, 0, 2 * self.boundary_size)
    }

    pub fn inland_origin(&self) -> IVec3 {
        self.origin + ivec3(self.boundary_size as i32, 0, self.boundary_size as i32)
    }

    pub fn center(&self) -> Vec3 {
        self.origin.as_vec3() + (0.5 * self.full_dimensions().as_vec3())
    }
}

pub type RegionId = u32;

// Debug viz stuff

#[derive(Component)]
pub struct IslandBoundsDebugViz;

pub fn island_bounds_debug_viz(
    mut gizmos: Gizmos,
    _debug_viz_query: Single<&IslandBoundsDebugViz>,
    island_gen: Single<&IslandGenerator>,
) {
    use bevy::color::palettes::css::{BLACK, GRAY, PURPLE};

    let island_origin = island_gen.origin;
    let island_size = island_gen.full_dimensions();
    let island_scale = island_size.as_vec3();
    let island_center = island_gen.center();

    gizmos.circle(
        Isometry3d::new(island_origin.as_vec3(), Quat::from_rotation_x(0.5 * 3.14)),
        0.5,
        PURPLE,
    );

    gizmos.cuboid(
        Transform::from_translation(island_center).with_scale(island_scale),
        BLACK,
    );

    let inland_scale = island_gen.inland_dimensions.as_vec3();

    gizmos.cuboid(
        Transform::from_translation(island_center).with_scale(inland_scale),
        GRAY,
    );
}

#[derive(Component)]
pub struct ThirdPassRegionDebugViz;

pub fn third_pass_region_debug_viz(
    mut gizmos: Gizmos,
    region_graph_query: Single<(&Regions<SecondPassRegion>, &RegionEdges, &PathEdges)>,
    _debug_viz_query: Single<&ThirdPassRegionDebugViz>,
) {
    use FeatureKind::*;
    use bevy::color::palettes::css::{BLACK, BLUE, DARK_BLUE, GRAY, RED, WHITE, YELLOW};

    let (Regions(regions), RegionEdges(region_edges), PathEdges(path_edges)) = *region_graph_query;

    for region in regions.values() {
        let color = match region.feature_kind {
            Required => BLUE,
            Bonus => YELLOW,
            Dangerous => RED,
            NoFeature => GRAY,
        };

        gizmos.circle(
            Isometry3d::from_translation(
                vec3(region.center.x as f32, 1.0, region.center.y as f32) + Vec3::splat(0.5),
            ),
            0.5,
            color,
        );
    }

    for (region_id, neighbor_ids) in region_edges {
        for neighbor_id in neighbor_ids {
            if region_id < neighbor_id {
                let region = &regions[region_id];
                let neighbor = &regions[neighbor_id];

                let edge = if region_id <= neighbor_id {
                    (*region_id, *neighbor_id)
                } else {
                    (*neighbor_id, *region_id)
                };

                let color = if path_edges.contains(&edge) {
                    DARK_BLUE
                } else {
                    BLACK
                };

                gizmos.line(
                    vec3(region.center.x as f32, 1.0, region.center.y as f32),
                    vec3(neighbor.center.x as f32, 1.0, neighbor.center.y as f32),
                    color,
                );
            }
        }
    }
}

#[derive(Component)]
pub struct PathPlacementDebugViz;

pub fn init_path_placement_debug_viz(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _debug_viz_query: Single<(Entity, &PathPlacementDebugViz)>,
    island_gen_query: Single<(&IslandGenerator, &Regions<SecondPassRegion>, &PathPlacement)>,
) {
    let (island_gen, Regions(regions), PathPlacement(path_cells)) = *island_gen_query;

    let image_size = island_gen.full_dimensions().xz();
    let mut image_data = Vec::with_capacity(4 * (image_size.x * image_size.y) as usize);

    for y in 0..image_size.y {
        for x in 0..image_size.x {
            let map_offset = uvec2(x, y);
            let map_pos = island_gen.origin.xz() + map_offset.as_ivec2();

            let sample_pos = map_pos.as_vec2() + Vec2::splat(0.5);
            let region_id: u32 = island_gen.region_id_noise.sample(sample_pos);

            if path_cells.contains_key(&map_pos) {
                image_data.push(125);
                image_data.push(0);
                image_data.push(250);
                image_data.push(255);
            } else if regions.contains_key(&region_id) {
                image_data.push(0);
                image_data.push((region_id >> 25) as u8 + 10);
                image_data.push(0);
                image_data.push(255);
            } else {
                image_data.push((region_id >> 25) as u8 + 10);
                image_data.push(0);
                image_data.push(0);
                image_data.push(255);
            }
        }
    }

    let plane = meshes.add(Plane3d::new(
        Vec3::Y,
        0.5 * island_gen.full_dimensions().xz().as_vec2(),
    ));

    let image = images.add(Image::new(
        Extent3d {
            width: image_size.x,
            height: image_size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::all(),
    ));

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image),
        unlit: true,
        // alpha_mode: AlphaMode::Mask(1.0),
        ..default()
    });

    commands.spawn((
        Transform::from_translation(
            island_gen.center() - 0.5 * Vec3::Y * island_gen.full_dimensions().y as f32,
        ),
        Mesh3d(plane),
        MeshMaterial3d(material),
    ));
}
