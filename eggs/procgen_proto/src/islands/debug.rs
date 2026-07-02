/// Some rudimentary visualizations of the island generation process, mainly for debugging purposes.
// TODO: only include this in dev builds
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::islands::*;

#[derive(Component)]
pub struct IslandBoundsDebugViz;

pub fn island_bounds_debug_viz(
    mut gizmos: Gizmos,
    _debug_viz_query: Single<&IslandBoundsDebugViz>,
    island_gen: Single<&IslandGenerator>,
) {
    use bevy::color::palettes::css::{BLACK, GRAY, PURPLE};

    // let island_origin = island_gen.origin;
    let island_size = island_gen.full_dimensions();
    let island_scale = island_size.as_vec3();
    let island_center = island_gen.center();

    // gizmos.circle(
    //     Isometry3d::new(island_origin.as_vec3(), Quat::from_rotation_x(0.5 * 3.14)),
    //     0.5,
    //     PURPLE,
    // );

    gizmos.cube(
        Transform::from_translation(island_center).with_scale(island_scale),
        BLACK,
    );

    let inland_scale = island_gen.inland_dimensions.as_vec3();

    gizmos.cube(
        Transform::from_translation(island_center).with_scale(inland_scale),
        GRAY,
    );
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

pub fn create_path_placement_debug_viz(
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
            let map_pos = /*island_gen.origin.xz() +*/ map_offset; //.as_ivec2();

            let sample_pos = map_pos.as_vec2() + Vec2::splat(0.5);
            let region_id: u32 = island_gen.region_id_noise.sample(sample_pos);
            let value = (region_id >> 25) as u8 + 10;

            if let Some(region) = regions.get(&region_id) {
                if map_pos == region.center {
                    image_data.push(240);
                    image_data.push(240);
                    image_data.push(240);
                    image_data.push(255);
                } else if path_cells.contains_key(&map_pos) {
                    image_data.push(125);
                    image_data.push(0);
                    image_data.push(250);
                    image_data.push(255);
                } else {
                    match region.feature_kind {
                        FeatureKind::Required => {
                            image_data.push(0);
                            image_data.push(0);
                            image_data.push(value);
                            image_data.push(255);
                        }
                        FeatureKind::Bonus => {
                            image_data.push(0);
                            image_data.push(value);
                            image_data.push(0);
                            image_data.push(255);
                        }
                        FeatureKind::Dangerous => {
                            image_data.push(value);
                            image_data.push(0);
                            image_data.push(0);
                            image_data.push(255);
                        }
                        FeatureKind::NoFeature => {
                            image_data.push(value / 2);
                            image_data.push(value / 2);
                            image_data.push(0);
                            image_data.push(255);
                        }
                    }
                }
            } else {
                image_data.push(value);
                image_data.push(value);
                image_data.push(value);
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
