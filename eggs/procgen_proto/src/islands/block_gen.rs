use bevy::prelude::*;

use crate::islands::*;

pub fn generate_surface_blocks(
    mut commands: Commands,
    island_gen_query: Single<(&IslandGenerator, &Regions<SecondPassRegion>, &PathPlacement)>,
) {
    info!("gen surface blocks");

    let (island_gen, Regions(regions), PathPlacement(path_cells)) = *island_gen_query;

    let island_size = island_gen.full_dimensions();
    // let island_center = island_gen.center();
    let sea_level_ratio = 0.75;

    let mut surface_heights: Vec<Vec<Option<i32>>> =
        vec![vec![None; island_size.z as usize]; island_size.x as usize];

    for z in 0..island_size.z {
        let z_edge_distance = z.min(island_size.z - z);

        for x in 0..island_size.x {
            let map_pos = uvec2(x, z);
            let sample_pos = map_pos.as_vec2() + Vec2::splat(0.5);

            let mut path_distance: Option<f32> = None;

            // this is probably not too efficient, but fine for now unless island gen gets noticeably slow
            for dz in -3..=3 {
                for dx in -3..=3 {
                    let offset_pos = map_pos.saturating_add_signed(ivec2(dx, dz));
                    let offset_distance = vec2(dx as f32, dz as f32).length();

                    if path_cells.contains_key(&offset_pos) {
                        path_distance = if let Some(dist) = path_distance {
                            Some(dist.min(offset_distance))
                        } else {
                            Some(offset_distance)
                        };
                    }
                }
            }

            let high_freq_weight = if let Some(dist) = path_distance {
                dist / Vec2::splat(3.0).length()
            } else {
                1.0
            };

            let smooth_elevation: f32 = island_gen.smooth_elevation_noise.sample(sample_pos);
            let high_freq_elevation: f32 = island_gen.high_freq_elevation_noise.sample(sample_pos);
            let base_elevation = (1.0 - high_freq_weight) * smooth_elevation
                + (high_freq_weight * high_freq_elevation);

            let x_edge_distance = x.min(island_size.x - x);
            let edge_distance = x_edge_distance.min(z_edge_distance);
            let scaled_edge_distance = (edge_distance.min(island_gen.boundary_size) as f32)
                / (island_gen.boundary_size as f32);

            let elevation = base_elevation + scaled_edge_distance.sqrt() - 1.0;

            if elevation >= 0.0 {
                let sea_level = sea_level_ratio * (island_size.y as f32);
                let elevation_scale = (1.0 - sea_level_ratio) * (island_size.y as f32);
                let y = sea_level + elevation * elevation_scale;
                let block_pos = ivec3(x as i32, y as i32, z as i32);

                if path_distance == Some(0.0) {
                    commands.spawn((PathBlock, GridPos(block_pos)));
                } else {
                    commands.spawn((GrassBlock, GridPos(block_pos)));
                }

                surface_heights[x as usize][z as usize] = Some(y as i32);
            }
        }
    }

    // spawn "cliff" blocks to fill in gaps where height changes by more than 1 between adjacent blocks
    for z in 0..island_size.z {
        let min_dz = if z == 0 { 0 } else { -1 };
        let max_dz = if z == island_size.z - 1 { 0 } else { 1 };

        for x in 0..island_size.x {
            if let Some(y) = surface_heights[x as usize][z as usize] {
                let mut min_neighbor_y = None;

                let min_dx = if x == 0 { 0 } else { -1 };
                let max_dx = if x == island_size.x - 1 { 0 } else { 1 };

                // TODO this is checking more neighbors than necessary? (diagonals)
                for dz in min_dz..=max_dz {
                    for dx in min_dx..=max_dx {
                        if let Some(neighbor_y) = surface_heights
                            [x.saturating_add_signed(dx) as usize]
                            [z.saturating_add_signed(dz) as usize]
                        {
                            if neighbor_y < y {
                                min_neighbor_y =
                                    Some(min_neighbor_y.unwrap_or(i32::MAX).min(neighbor_y));
                            }
                        }
                    }
                }

                if let Some(min_y) = min_neighbor_y {
                    for cliff_y in (min_y + 1)..y {
                        let block_pos = /*island_gen.origin +*/ ivec3(x as i32, cliff_y, z as i32);

                        if y - cliff_y < 4 {
                            commands.spawn((DirtBlock, GridPos(block_pos)));
                        } else {
                            commands.spawn((StoneBlock, GridPos(block_pos)));
                        }
                    }
                }
            }
        }
    }

    info!("blocks spawned");
}
