use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use noiz::SampleableFor;

use crate::islands::{IslandGenerator, RegionId};

#[derive(Component)]
pub struct Regions<R: Region>(pub HashMap<RegionId, R>);

trait Region {}

pub struct FirstPassRegion {
    pub area: u32,
    pub center: IVec2,
    pub center_dist: f32,
    pub is_inland: bool,
}

impl Region for FirstPassRegion {}

pub struct SecondPassRegion {
    pub area: u32,
    pub center: IVec2,
    pub feature_kind: FeatureKind,

    // offset of the placed feature relative to the center of the region
    pub feature_offset: IVec2,
}

impl Region for SecondPassRegion {}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureKind {
    Required,
    Bonus,
    Dangerous,
    NoFeature,
    // BoundaryRegion,
    // TinyRegion,
}

#[derive(Component)]
pub struct FeatureRegions(pub HashMap<FeatureKind, Vec<RegionId>>);

#[derive(Component)]
pub struct RegionEdges(pub HashMap<RegionId, HashSet<RegionId>>);

const NUM_REQUIRED_FEATURES: u32 = 3;
const NUM_DANGEROUS_FEATURES: u32 = 3;
const NUM_BONUS_FEATURES: u32 = 5;
pub const TOTAL_NUM_FEATURES: u32 =
    NUM_REQUIRED_FEATURES + NUM_DANGEROUS_FEATURES + NUM_BONUS_FEATURES;

// A region must contain at least 10 surface cells to be allowed to contain a feature
const REGION_AREA_THRESHOLD: u32 = 10;

// First pass initializes region graphs and evaluates noise (should it? or would it be better to wait and save storage?)
pub fn generate_first_pass_region_graph(
    mut commands: Commands,
    island_gen_query: Single<(Entity, &IslandGenerator)>,
) {
    info!("1st pass graph");

    let (island_gen_entity, island_gen) = *island_gen_query;

    let mut regions: HashMap<RegionId, FirstPassRegion> = HashMap::new();
    let mut region_neighbors: HashMap<RegionId, HashSet<RegionId>> = HashMap::new();

    let region_id_noise = island_gen.region_id_noise;
    let region_center_distance_noise = island_gen.region_center_distance_noise;

    let island_origin = island_gen.origin;
    let island_size = island_gen.full_dimensions();

    let map_size = island_size.xz();

    let mut forbidden_region_ids = HashSet::new();

    for y in 0..map_size.y {
        let is_y_inland = y.min(map_size.y - y) > island_gen.boundary_size;

        for x in 0..map_size.x {
            let map_offset = ivec2(x as i32, y as i32);
            let map_pos = island_origin.xz() + map_offset;
            let sample_pos = map_pos.as_vec2() + Vec2::splat(0.5);

            let is_x_inland = x.min(map_size.x - x) > island_gen.boundary_size;
            let is_inland_cell = is_x_inland && is_y_inland;

            let region_id: u32 = region_id_noise.sample(sample_pos);
            let region_center_distance = region_center_distance_noise.sample(sample_pos);

            if !is_inland_cell {
                forbidden_region_ids.insert(region_id);
            } else {
                if let Some(region) = regions.get_mut(&region_id) {
                    region.area += 1;

                    if region_center_distance < region.center_dist {
                        region.center = map_pos;
                        region.center_dist = region_center_distance;
                    }

                    region.is_inland = region.is_inland && is_inland_cell;
                } else {
                    regions.insert(
                        region_id,
                        FirstPassRegion {
                            area: 1,
                            // center: cell_pos.xz(),
                            center: map_pos,
                            center_dist: region_center_distance,
                            is_inland: is_inland_cell,
                        },
                    );
                }

                if x > 0 {
                    // let left_neighbor_id = region_id_noise.sample(sample_pos - Vec3::X);
                    let left_neighbor_id: u32 = region_id_noise.sample(sample_pos - Vec2::X);

                    if left_neighbor_id != region_id
                        && !forbidden_region_ids.contains(&left_neighbor_id)
                    {
                        if let Some(local_neighbors) = region_neighbors.get_mut(&region_id) {
                            local_neighbors.insert(left_neighbor_id);
                        } else {
                            region_neighbors.insert(region_id, HashSet::from([left_neighbor_id]));
                        }

                        if let Some(other_neighbors) = region_neighbors.get_mut(&left_neighbor_id) {
                            other_neighbors.insert(region_id);
                        } else {
                            region_neighbors.insert(left_neighbor_id, HashSet::from([region_id]));
                        }
                    }
                }

                if y > 0 {
                    let upper_neighbor_id: u32 = region_id_noise.sample(sample_pos - Vec2::Y);

                    if upper_neighbor_id != region_id
                        && !forbidden_region_ids.contains(&upper_neighbor_id)
                    {
                        if let Some(local_neighbors) = region_neighbors.get_mut(&region_id) {
                            local_neighbors.insert(upper_neighbor_id);
                        } else {
                            region_neighbors.insert(region_id, HashSet::from([upper_neighbor_id]));
                        }

                        if let Some(other_neighbors) = region_neighbors.get_mut(&upper_neighbor_id)
                        {
                            other_neighbors.insert(region_id);
                        } else {
                            region_neighbors.insert(upper_neighbor_id, HashSet::from([region_id]));
                        }
                    }

                    // if x > 0 {
                    //     let upper_left_neighbor_id = region_id_noise.sample(sample_pos - Vec2::ONE);

                    //     if upper_left_neighbor_id != region_id && !forbidden_region_ids.contains(&upper_left_neighbor_id) {
                    //         if let Some(local_neighbors) = region_neighbors.get_mut(&region_id) {
                    //             local_neighbors.insert(upper_left_neighbor_id);
                    //         } else {
                    //             region_neighbors.insert(region_id, HashSet::from([upper_left_neighbor_id]));
                    //         }

                    //         if let Some(other_neighbors) = region_neighbors.get_mut(&upper_left_neighbor_id) {
                    //             other_neighbors.insert(region_id);
                    //         } else {
                    //             region_neighbors.insert(upper_left_neighbor_id, HashSet::from([region_id]));
                    //         }
                    //     }
                    // }

                    // if x < map_size.x - 1 {
                    //     let upper_right_neighbor_id = region_id_noise.sample(sample_pos + vec2(1.0, -1.0));

                    //     if upper_right_neighbor_id != region_id && !forbidden_region_ids.contains(&upper_right_neighbor_id) {
                    //         if let Some(local_neighbors) = region_neighbors.get_mut(&region_id) {
                    //             local_neighbors.insert(upper_right_neighbor_id);
                    //         } else {
                    //             region_neighbors.insert(region_id, HashSet::from([upper_right_neighbor_id]));
                    //         }

                    //         if let Some(other_neighbors) = region_neighbors.get_mut(&upper_right_neighbor_id) {
                    //             other_neighbors.insert(region_id);
                    //         } else {
                    //             region_neighbors.insert(upper_right_neighbor_id, HashSet::from([region_id]));
                    //         }
                    //     }
                    // }
                }
            }
        }
    }

    regions.retain(|key, _value| !forbidden_region_ids.contains(key));

    region_neighbors.retain(|key, _value| !forbidden_region_ids.contains(key));

    for neighbors in region_neighbors.values_mut() {
        neighbors.retain(|region_id| !forbidden_region_ids.contains(region_id));
    }

    commands
        .entity(island_gen_entity)
        .insert((Regions(regions), RegionEdges(region_neighbors)));
}

pub fn generate_second_pass_region_graph(
    mut commands: Commands,
    island_gen_query: Single<(
        Entity,
        &IslandGenerator,
        &Regions<FirstPassRegion>,
        &RegionEdges,
    )>,
) {
    info!("2nd pass graph");

    use FeatureKind::*;

    let (island_gen_entity, _island_gen, Regions(regions), RegionEdges(_region_edges)) =
        *island_gen_query;

    let mut required_feature_count = 0;
    let mut dangerous_feature_count = 0;
    let mut bonus_feature_count = 0;
    // let rng = NoiseRng(island_gen.seed);

    let mut second_pass_regions = HashMap::with_capacity(regions.len());
    let mut feature_regions: HashMap<FeatureKind, Vec<RegionId>> =
        HashMap::with_capacity(TOTAL_NUM_FEATURES as usize);

    // TODO: this is a bad way to place features!
    for (region_id, region) in regions {
        let selected_feature = if region.area < REGION_AREA_THRESHOLD {
            // TinyRegion
            NoFeature
        } else {
            if required_feature_count < NUM_REQUIRED_FEATURES {
                required_feature_count += 1;

                feature_regions
                    .entry(Required)
                    .and_modify(|required_features| required_features.push(*region_id))
                    .or_insert(Vec::with_capacity(NUM_REQUIRED_FEATURES as usize));

                Required
            } else if dangerous_feature_count < NUM_DANGEROUS_FEATURES {
                dangerous_feature_count += 1;

                feature_regions
                    .entry(Dangerous)
                    .and_modify(|dangerous_features| dangerous_features.push(*region_id))
                    .or_insert(Vec::with_capacity(NUM_DANGEROUS_FEATURES as usize));

                Dangerous
            } else if bonus_feature_count < NUM_BONUS_FEATURES {
                // TODO don't hardcode
                bonus_feature_count += 1;

                feature_regions
                    .entry(Bonus)
                    .and_modify(|bonus_features| bonus_features.push(*region_id))
                    .or_insert(Vec::with_capacity(NUM_BONUS_FEATURES as usize));

                Bonus
            } else {
                NoFeature
            }
        };

        second_pass_regions.insert(
            *region_id,
            SecondPassRegion {
                area: region.area,
                center: region.center,
                feature_kind: selected_feature,
                feature_offset: IVec2::ZERO, // TODO!!!
            },
        );
    }

    // TODO: need to actually handle these scenarios
    debug_assert_eq!(required_feature_count, NUM_REQUIRED_FEATURES);
    debug_assert!(dangerous_feature_count <= NUM_DANGEROUS_FEATURES);

    commands.entity(island_gen_entity).insert((
        Regions(second_pass_regions),
        FeatureRegions(feature_regions),
    ));
}

#[derive(Component)]
pub struct SecondPassRegionDebugViz;

pub fn second_pass_region_debug_viz(
    mut gizmos: Gizmos,
    _debug_viz_query: Single<&SecondPassRegionDebugViz>,
    region_graph_query: Single<(&Regions<SecondPassRegion>, &RegionEdges)>,
) {
    use FeatureKind::*;
    use bevy::color::palettes::css::{BLACK, BLUE, GREEN, RED, WHITE};

    let (Regions(regions), RegionEdges(region_edges)) = *region_graph_query;

    for region in regions.values() {
        let color = match region.feature_kind {
            Required => BLUE,
            Bonus => GREEN,
            Dangerous => RED,
            NoFeature => WHITE,
        };

        gizmos.circle(
            Isometry3d::from_translation(vec3(region.center.x as f32, 0.0, region.center.y as f32)),
            0.5,
            color,
        );
    }

    for (region_id, neighbor_ids) in region_edges {
        for neighbor_id in neighbor_ids {
            if region_id < neighbor_id {
                let region = &regions[region_id];
                let neighbor = &regions[neighbor_id];

                gizmos.line(
                    vec3(region.center.x as f32, 0.0, region.center.y as f32),
                    vec3(neighbor.center.x as f32, 0.0, neighbor.center.y as f32),
                    BLACK,
                );
            }
        }
    }
}
