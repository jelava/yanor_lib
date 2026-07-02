use core::f32;

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::islands::*;

#[derive(Component)]
pub struct PathEdges(pub HashSet<(RegionId, RegionId)>);

/// Stores cells where paths between features are
#[derive(Component)]
pub struct PathPlacement(pub HashMap<UVec2, PathKind>);

pub enum PathKind {
    Road,
}

pub fn generate_path_edges(
    mut commands: Commands,
    island_gen_query: Single<(
        Entity,
        &IslandGenerator,
        &Regions<SecondPassRegion>,
        &RegionEdges,
        &FeatureRegions,
    )>,
) {
    info!("choose path edges");

    let (
        island_gen_entity,
        island_gen,
        Regions(regions),
        RegionEdges(region_edges),
        FeatureRegions(feature_regions),
    ) = *island_gen_query;

    let rng = NoiseRng(island_gen.seed);
    let mut rng_input = 0;

    let mut walk_vertex = *regions
        .keys()
        .next()
        .expect("There should be at least one region..."); // TODO: don't do this

    // These are the regions that must be connected via paths
    let mut destination_regions = feature_regions[&FeatureKind::Required].clone();

    info!("{}", destination_regions.len());

    let mut path_edges = HashSet::<(RegionId, RegionId)>::new();
    // let mut connected_feature_regions = HashSet::<RegionId>::new();

    // walk around the region graph until every region with a feature is connected
    // while connected_feature_regions.len() < TOTAL_NUM_FEATURES as usize {
    while !destination_regions.is_empty() {
        // info!("{}", destination_regions.len());
        // info!("walking at: {walk_vertex}");

        if let Some(neighbors) = region_edges.get(&walk_vertex) {
            let mut next_vertex = *neighbors.iter().next().unwrap(); // TODO: don't unwrap!

            let current_pos = regions[&walk_vertex].center.as_vec2();
            let mut current_score = f32::MAX;
            // let mut best_score = f32::MAX;

            for destination_region_id in &destination_regions {
                let destination_pos = &regions[destination_region_id].center.as_vec2();
                current_score = current_score.min(destination_pos.distance_squared(current_pos));
            }

            for neighbor_id in neighbors {
                let neighbor = &regions[neighbor_id];
                let neighbor_pos = neighbor.center.as_vec2();
                let mut neighbor_score = f32::MAX;

                if destination_regions.contains(neighbor_id) {
                    next_vertex = *neighbor_id;
                    destination_regions.remove(&next_vertex);

                    info!("dest found: {neighbor_id}");
                    info!("{}", destination_regions.len());

                    break;
                } else {
                    for destination_region_id in &destination_regions {
                        let destination_pos = &regions[destination_region_id].center.as_vec2();
                        neighbor_score =
                            neighbor_score.min(destination_pos.distance_squared(neighbor_pos));
                    }

                    let r1 = rng.rand_u32(rng_input) >> 15;
                    let r2 = rng.rand_u32(rng_input + 1) >> 15;
                    rng_input += 2;

                    if neighbor_score < current_score + (r1 as f32) - (r2 as f32) {
                        // info!("{}", neighbor_score - current_score);
                        next_vertex = *neighbor_id;
                        break;
                    }
                }
            }

            // Randomly choose between picking the next step in the walk randomly or using a distance-based heuristic
            // Always having a chance to pick some steps randomly will break the heuristic-based walk out of cycles
            // let walk_rng = rng.rand_u32(rng_input) >> 30;
            // rng_input += 1;

            // if walk_rng != 0 {
            //     let current_pos = regions[&walk_vertex].center.as_vec2();
            //     let mut best_score = f32::MAX;

            //     for neighbor_id in neighbors {
            //         let neighbor = &regions[neighbor_id];

            //         // If the neighboring region contains a feature that hasn't already been visited, visit it
            //         if neighbor.feature_kind != FeatureKind::NoFeature
            //             && !connected_feature_regions.contains(neighbor_id)
            //         {
            //             next_vertex = *neighbor_id;
            //             break;
            //         }

            //         let mut score = f32::MAX; //(rng.rand_u32(rng_input) >> 20) as f32;

            //         for region_id in feature_regions.values().flatten() {
            //             let feature_pos = regions[region_id].center.as_vec2();
            //             score = current_pos.distance(feature_pos).min(score);
            //         }

            //         // score += (rng.rand_u32(rng_input) >> 20) as f32;
            //         // rng_input += 1;

            //         if score < best_score {
            //             next_vertex = *neighbor_id;
            //             best_score = score;
            //         }
            //     }
            // } else {
            //     let index = ((rng.rand_u32(rng_input) >> 8) as usize) % neighbors.len();
            //     rng_input = rng_input.wrapping_add(1);

            //     // TODO: better way to pick next step in walk then random neighbor?
            //     next_vertex = *neighbors.iter().nth(index).unwrap();
            // }

            // if regions[&next_vertex].feature_kind != FeatureKind::NoFeature {
            //     connected_feature_regions.insert(next_vertex);
            // }

            debug_assert_ne!(walk_vertex, next_vertex);

            let edge = if walk_vertex < next_vertex {
                (walk_vertex, next_vertex)
            } else {
                (next_vertex, walk_vertex)
            };

            path_edges.insert(edge);
            walk_vertex = next_vertex;
        } else {
            // TODO: this shouldn't happen unless something weird happens earlier in the gen process, but still handle it better
            panic!("region {walk_vertex} has no neighbors? walk stuck!");
        }
    }

    commands
        .entity(island_gen_entity)
        .insert(PathEdges(path_edges));
}

pub fn generate_path_placements(
    mut commands: Commands,
    island_gen_query: Single<(
        Entity,
        &IslandGenerator,
        &Regions<SecondPassRegion>,
        &PathEdges,
    )>,
) {
    info!("place paths");

    let (island_gen_entity, island_gen, Regions(regions), PathEdges(path_edges)) =
        *island_gen_query;

    let elevation_noise = island_gen.smooth_elevation_noise;
    let rng = NoiseRng(island_gen.seed);
    let mut rng_input = 0;

    let mut path_cells = HashMap::new();

    for (start_region_id, end_region_id) in path_edges {
        let start_region = &regions[start_region_id];
        let start_pos = start_region
            .center
            .saturating_add_signed(start_region.feature_offset);

        let end_region = &regions[end_region_id];
        let end_pos = end_region
            .center
            .saturating_add_signed(end_region.feature_offset);

        let mut path_pos = start_pos;

        // TODO: calculate based on island size/"sea level"
        let elevation_scale = 1.0;

        path_cells.insert(start_pos, PathKind::Road);

        while path_pos != end_pos {
            let path_elevation_sample: f32 =
                elevation_noise.sample(path_pos.as_vec2() + Vec2::splat(0.5));
            let path_elevation = elevation_scale * path_elevation_sample;

            let neighbors = [
                path_pos + UVec2::X,
                path_pos - UVec2::X,
                path_pos + UVec2::Y,
                path_pos - UVec2::Y,
            ];

            // each neighbor is given a "score" which is used to prioritize where to walk next, lower scores are better
            let mut best_score = f32::MAX;
            let mut best_pos = neighbors[0];

            for neighbor_pos in neighbors {
                if neighbor_pos == end_pos {
                    best_pos = neighbor_pos;
                    break;
                }

                let neighbor_region_id: u32 = island_gen
                    .region_id_noise
                    .sample(neighbor_pos.as_vec2() + Vec2::splat(0.5));

                if regions.contains_key(&neighbor_region_id) {
                    let dist_to_end = neighbor_pos.as_vec2().distance(end_pos.as_vec2()); //(neighbor_pos.distance_squared(end_pos) as f32).sqrt();

                    let neighbor_elevation_sample: f32 =
                        elevation_noise.sample(neighbor_pos.as_vec2() + Vec2::splat(0.5));
                    let neighbor_elevation = elevation_scale * neighbor_elevation_sample;

                    let elevation_change = (neighbor_elevation - path_elevation) * 15.0;

                    let r1 = 0.0625 * (rng.rand_u32(rng_input) >> 28) as f32;
                    let r2 = 0.0625 * (rng.rand_u32(rng_input + 1) >> 28) as f32 - 0.5;
                    rng_input += 2;

                    let elevation_change = (neighbor_elevation - path_elevation) * r1 * 30.0;

                    // add some extra randomness to make paths less boring?
                    // let mut score = dist_to_end + elevation_change + r1 - r2;
                    let mut score = dist_to_end + elevation_change + r2;

                    if path_cells.contains_key(&neighbor_pos) {
                        score *= 1.1;
                    }

                    if score < best_score {
                        best_score = score;
                        best_pos = neighbor_pos;
                    }
                }
            }

            path_pos = best_pos;
            path_cells.insert(path_pos, PathKind::Road);
        }
    }

    commands
        .entity(island_gen_entity)
        .insert(PathPlacement(path_cells));
}
