use bevy::{camera::primitives::Aabb, prelude::*};

use crate::islands::{ChunkedIslandGenerator, Island, IslandGenerationState};

#[derive(Component, Clone, Copy, Default)]
#[require(Transform, Visibility)]
pub struct IslandChunkGenerator {
    pub seed: u32,
}

// Each chunk is a square slice of space containing this many cells along the X and Z axes
pub const ISLAND_CHUNK_SIZE: usize = 16;

#[derive(Component, Clone, Default)]
#[require(Transform, Visibility)]
pub struct IslandChunk {
    pub terrain_map: Vec<Terrain>,
}

#[derive(Clone, Copy, Default)]
pub struct Terrain {
    /// Height of the terrain within the chunk (i.e. relative to chunk transform)
    pub chunk_height: u32,
    pub background: TerrainBackground,
    pub foreground: TerrainForeground,
}

#[derive(Clone, Copy, Default)]
pub enum TerrainBackground {
    #[default]
    Dirt,
    Grass,
    Snow,
    Stone,
}

#[derive(Clone, Copy, Default)]
pub enum TerrainForeground {
    #[default]
    Empty,
    StonePath,
}

pub trait Generate<C = ()> {
    fn generate(&self, context: C) -> impl Scene;
}

impl Generate<Entity> for ChunkedIslandGenerator {
    fn generate(&self, context: Entity) -> impl Scene {
        let island_gen_entity = context;
        let mut chunk_gens = Vec::new();
        let mut chunk_seed = self.seed;

        for chunk_y in 0..self.island_size.y {
            for chunk_x in 0..self.island_size.x {
                let chunk_pos = (ISLAND_CHUNK_SIZE as u32) * uvec3(chunk_x, 0, chunk_y);

                chunk_gens.push(bsn! {
                    #island_chunk_generator
                    IslandChunkGenerator { seed: chunk_seed }
                    Transform::from_translation(chunk_pos.as_vec3())
                });

                chunk_seed += 1;
            }
        }

        bsn! {
            #island
            Island { seed: {self.seed} }
            ChildOf(island_gen_entity)
            Children [ {chunk_gens} ]
        }
    }
}

pub fn spawn_chunk_generators(
    mut commands: Commands,
    mut next_island_gen_state: ResMut<NextState<IslandGenerationState>>,
    island_gen_query: Single<(Entity, &ChunkedIslandGenerator)>,
) {
    let (island_gen_entity, island_gen) = *island_gen_query;
    commands.spawn_scene(island_gen.generate(island_gen_entity));
    next_island_gen_state.set(IslandGenerationState::GenerateChunks);
}

impl Generate<Entity> for IslandChunkGenerator {
    fn generate(&self, context: Entity) -> impl Scene {
        let mut terrain_map = Vec::with_capacity(ISLAND_CHUNK_SIZE * ISLAND_CHUNK_SIZE);

        for z in 0..ISLAND_CHUNK_SIZE {
            for x in 0..ISLAND_CHUNK_SIZE {
                // TODO: calculate/generate actual values here
                terrain_map.push(Terrain {
                    chunk_height: 0,
                    background: TerrainBackground::default(),
                    foreground: TerrainForeground::default(),
                });
            }
        }

        bsn! {
            #island_chunk
            IslandChunk { terrain_map }
            ChildOf(context)
        }
    }
}

pub fn generate_chunks(
    mut commands: Commands,
    // island_gen_query: Single<&ChunkedIslandGenerator>,
    chunk_gen_query: Query<(Entity, &IslandChunkGenerator)>,
) {
    for (chunk_gen_entity, chunk_gen) in &chunk_gen_query {
        commands.spawn_scene(chunk_gen.generate(chunk_gen_entity));

        // TODO: don't do this?
        commands
            .entity(chunk_gen_entity)
            .remove::<IslandChunkGenerator>();
    }
}
