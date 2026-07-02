/// Procedural generation of floating islands
pub mod block_gen;
pub mod chunk_gen;
pub mod debug;
pub mod path_gen;
pub mod region_graph;

use bevy::prelude::*;

use noiz::{
    Noise, SampleableFor,
    cell_noise::PerNearestPoint,
    cells::Voronoi,
    lengths::HybridLength,
    prelude::{
        FractalLayers, LayeredNoise, Normed, Octave, PerCellPointDistances, Persistence,
        SNormToUNorm, WorleyLeastDistance, common_noise::Simplex,
    },
    rng::{NoiseRng, Random},
};

use yanor_core::grid::GridPos;

use crate::{
    blocks::{DirtBlock, GrassBlock, PathBlock, StoneBlock},
    islands::{
        block_gen::*, chunk_gen::*, debug::create_path_placement_debug_viz, path_gen::*,
        region_graph::*,
    },
    noiz_ext::FullRange,
    presentation::islands::generate_surface_mesh,
};

pub struct IslandPlugin;

impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        // TODO: use states/custom schedule/something to control when island gen happens rather than explicitly tying to Startup
        app.add_systems(
            Startup,
            (
                generate_first_pass_region_graph,
                generate_second_pass_region_graph,
                generate_path_edges,
                generate_path_placements,
                // generate_surface_blocks,
                generate_surface_mesh,
            )
                .chain(),
        );
        // .add_systems(PostStartup, create_path_placement_debug_viz);
    }
}

pub struct ChunkedIslandPlugin;

impl Plugin for ChunkedIslandPlugin {
    fn build(&self, app: &mut App) {
        use IslandGenerationState::*;

        app.init_state::<IslandGenerationState>()
            .add_systems(OnEnter(SpawnChunkGenerators), spawn_chunk_generators)
            .add_systems(
                FixedUpdate,
                generate_chunks.run_if(in_state(GenerateChunks)),
            );
    }
}

#[derive(Component, Clone, Copy, Default)]
#[require(Transform, Visibility)]
pub struct Island {
    pub seed: u32,
}

#[derive(States, Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub enum IslandGenerationState {
    #[default]
    NotStarted,
    SpawnChunkGenerators,
    GenerateChunks,
    //...?
    Finished,
}

/// This component drives the island generation process. Adding it to an entity will cause it to be
/// processed by island generation systems which will (eventually) spawn the actual island entities.
#[derive(Component)]
pub struct IslandGenerator {
    /// RNG seed for the island
    pub seed: u32,

    /// Size of the inner bounding box containing the "inland" area that's guaranteed to have solid ground
    pub inland_dimensions: UVec3,

    /// Size of the boundary area around each edge of the inner bounding box where the "shore" of the island is
    pub boundary_size: u32,

    /// Generates (most likely) unique IDs for each region
    pub region_id_noise: Noise<PerNearestPoint<Voronoi, HybridLength, Random<FullRange, u32>>>,

    /// Used to find the cell with the lowest distance to center in the region, which is considered it's "center"
    pub region_center_distance_noise:
        Noise<PerCellPointDistances<Voronoi, HybridLength, WorleyLeastDistance>>,

    /// Relatively smooth noise used to generate elevation for areas that are supposed to be traversable
    /// (i.e. along paths/roads)
    pub smooth_elevation_noise: Noise<(Simplex, SNormToUNorm)>,

    /// Higher frequency noise used for elevation in areas that aren't necessarily traversable, to allow for more
    /// variety in terrain
    pub high_freq_elevation_noise: Noise<(
        LayeredNoise<Normed<f32>, Persistence, FractalLayers<Octave<Simplex>>>,
        SNormToUNorm,
    )>,
}

const DEFAULT_NOISE_FREQUENCY: f32 = 0.08;
const DEFAULT_VORONOI_RANDOMNESS: f32 = 0.8;

impl IslandGenerator {
    pub fn new(seed: u32, inland_dimensions: UVec3, boundary_size: u32) -> Self {
        let voronoi = Voronoi::<false>::default_with_randomness(DEFAULT_VORONOI_RANDOMNESS);
        let length_mode = HybridLength;
        let seed_rng = NoiseRng(seed);

        Self {
            seed,
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
            smooth_elevation_noise: Noise {
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

    pub fn center(&self) -> Vec3 {
        0.5 * self.full_dimensions().as_vec3()
    }
}

pub type RegionId = u32;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct ChunkedIslandGenerator {
    /// Base RNG seed for the entire island
    pub seed: u32,

    /// Size of the island in chunks
    pub island_size: UVec2,
}
