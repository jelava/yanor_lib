use bevy::{
    asset::RenderAssetUsages,
    image::{ImageArrayLayout, ImageLoaderSettings},
    pbr::{ExtendedMaterial, wireframe::Wireframe},
    platform::collections::HashMap,
    prelude::*,
};

use noiz::SampleableFor;

use crate::{
    islands::{
        IslandGenerator,
        path_gen::{PathKind, PathPlacement},
    },
    presentation::materials::PixelArtMaterialExtension,
};

const LOWER_RIGHT: usize = 0;
const UPPER_RIGHT: usize = 1;
const UPPER_LEFT: usize = 2;
const LOWER_LEFT: usize = 3;

// Slightly higher-level intermediate representation of heightmap mesh geometry. The Vec3s in both Quad and Tri are the
// vertices of the shape, listed in counterclockwise order (relative to face normal) with the first vertex being the
// closest one to the origin
struct HeightmapMeshFace {
    geometry: HeightmapMeshGeometry,
    tile_kind: TileKind,
    terrain: Terrain,
}

enum HeightmapMeshGeometry {
    // Flat tiles and slopes/cliffs are all quads
    Quad(Vec3, Vec3, Vec3, Vec3),
    // Some corners consist of two triangles
    Tri(Vec3, Vec3, Vec3),
}

#[derive(Clone, Copy, Debug)]
enum TileKind {
    Center,
    EdgeX,
    EdgeZ,
    Corner,
}

#[derive(Clone, Copy, Debug)]
enum Terrain {
    Grass,
    GrassPath,
    DirtSlope,
    StoneSlope,
    Sand,
    SandPath,
    Snow,
    SnowPath,
    Mountain,
    MountainPath,
}

const TILE_LENGTH: f32 = 0.6875; // 22/32
const CORNER_LENGTH: f32 = 1.0 - TILE_LENGTH;

pub fn generate_surface_mesh(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, PixelArtMaterialExtension>>>,
    mut testmats: ResMut<Assets<StandardMaterial>>,
    island_gen_query: Single<(&IslandGenerator, &PathPlacement)>,
) {
    use {HeightmapMeshGeometry::*, Terrain::*, TileKind::*};

    let (island_gen, PathPlacement(path_cells)) = *island_gen_query;

    let island_size = island_gen.full_dimensions();
    let sea_level_ratio = 0.6; // TODO: don't hardcode!
    let sea_level = (sea_level_ratio * (island_size.y as f32)).round() as i32;

    let tile_size = vec3(TILE_LENGTH, 0.0, TILE_LENGTH);
    let corner_size = vec3(CORNER_LENGTH, 0.0, CORNER_LENGTH);
    let horizontal_edge_size = vec3(CORNER_LENGTH, 0.0, TILE_LENGTH);
    let vertical_edge_size = vec3(TILE_LENGTH, 0.0, CORNER_LENGTH);

    let mut heightmap_faces: Vec<HeightmapMeshFace> = Vec::new();

    for z_index in 0..(2 * island_size.z + 1) {
        let z_even = z_index % 2 == 0;

        for x_index in 0..(2 * island_size.x + 1) {
            let x_even = x_index % 2 == 0;
            let tile_component_pos = uvec2(x_index, z_index);
            let center_pos = 0.5 * vec2(x_index as f32, z_index as f32);

            if x_even && z_even {
                let tile_kind = Corner;

                let offsets = [ivec2(-1, -1), ivec2(-1, 1), ivec2(1, 1), ivec2(1, -1)];

                let maybe_tile_pos = offsets.map(|offset| {
                    tile_component_pos
                        .checked_add_signed(offset)
                        .and_then(|pos| Some(pos / 2))
                });

                let maybe_tile_heights = maybe_tile_pos.map(|maybe_pos| {
                    maybe_pos.and_then(|pos| calc_tile_height(pos, island_gen, path_cells))
                });

                let corner_offsets = offsets
                    .map(|offset| 0.5 * vec3(offset.x as f32, 0.0, offset.y as f32) * corner_size);

                let corners = [0, 1, 2, 3].map(|i| {
                    vec3(
                        center_pos.x,
                        maybe_tile_heights[i].unwrap_or(sea_level) as f32,
                        center_pos.y,
                    ) + corner_offsets[i]
                });

                let terrain = GrassPath; // TODO

                match maybe_tile_heights {
                    [None, None, None, None] => {}

                    [Some(_), None, None, None] => {
                        heightmap_faces.push(HeightmapMeshFace {
                            geometry: Tri(
                                corners[LOWER_RIGHT].into(),
                                corners[UPPER_RIGHT].into(),
                                corners[LOWER_LEFT].into(),
                            ),
                            tile_kind,
                            terrain,
                        });
                    }
                    [None, Some(_), None, None] => {
                        heightmap_faces.push(HeightmapMeshFace {
                            geometry: Tri(
                                corners[UPPER_RIGHT].into(),
                                corners[UPPER_LEFT].into(),
                                corners[LOWER_RIGHT].into(),
                            ),
                            tile_kind,
                            terrain,
                        });
                    }
                    [None, None, Some(_), None] => {
                        heightmap_faces.push(HeightmapMeshFace {
                            geometry: Tri(
                                corners[UPPER_LEFT].into(),
                                corners[LOWER_LEFT].into(),
                                corners[UPPER_RIGHT].into(),
                            ),
                            tile_kind,
                            terrain,
                        });
                    }
                    [None, None, None, Some(_)] => {
                        heightmap_faces.push(HeightmapMeshFace {
                            geometry: Tri(
                                corners[LOWER_LEFT].into(),
                                corners[LOWER_RIGHT].into(),
                                corners[UPPER_LEFT].into(),
                            ),
                            tile_kind,
                            terrain,
                        });
                    }

                    _ => {
                        let coplanarity_test = (corners[LOWER_LEFT] - corners[LOWER_RIGHT]).dot(
                            (corners[UPPER_RIGHT] - corners[LOWER_RIGHT])
                                .cross(corners[UPPER_LEFT] - corners[LOWER_RIGHT]),
                        );

                        if coplanarity_test == 0.0 {
                            heightmap_faces.push(HeightmapMeshFace {
                                geometry: Quad(
                                    corners[LOWER_RIGHT],
                                    corners[UPPER_RIGHT],
                                    corners[UPPER_LEFT],
                                    corners[LOWER_LEFT],
                                ),
                                tile_kind,
                                terrain,
                            });
                        } else {
                            let diagonal_1_distance =
                                corners[LOWER_RIGHT].distance(corners[UPPER_LEFT]);
                            let diagonal_2_distance =
                                corners[LOWER_LEFT].distance(corners[UPPER_RIGHT]);

                            if diagonal_1_distance <= diagonal_2_distance {
                                heightmap_faces.push(HeightmapMeshFace {
                                    geometry: Tri(
                                        corners[UPPER_RIGHT].into(),
                                        corners[UPPER_LEFT].into(),
                                        corners[LOWER_RIGHT].into(),
                                    ),
                                    tile_kind,
                                    terrain,
                                });

                                heightmap_faces.push(HeightmapMeshFace {
                                    geometry: Tri(
                                        corners[LOWER_LEFT].into(),
                                        corners[LOWER_RIGHT].into(),
                                        corners[UPPER_LEFT].into(),
                                    ),
                                    tile_kind,
                                    terrain,
                                });
                            } else {
                                heightmap_faces.push(HeightmapMeshFace {
                                    geometry: Tri(
                                        corners[LOWER_RIGHT].into(),
                                        corners[UPPER_RIGHT].into(),
                                        corners[LOWER_LEFT].into(),
                                    ),
                                    tile_kind,
                                    terrain,
                                });

                                heightmap_faces.push(HeightmapMeshFace {
                                    geometry: Tri(
                                        corners[UPPER_LEFT].into(),
                                        corners[LOWER_LEFT].into(),
                                        corners[UPPER_RIGHT].into(),
                                    ),
                                    tile_kind,
                                    terrain,
                                });
                            }
                        }
                    }
                }
            } else if x_even {
                let tile_kind = EdgeX;

                let tile_offsets = [ivec2(-1, 0), ivec2(1, 0)];

                let maybe_tile_pos = tile_offsets.map(|offset| {
                    tile_component_pos
                        .checked_add_signed(offset)
                        .and_then(|pos| Some(pos / 2))
                });

                let maybe_tile_heights = maybe_tile_pos.map(|maybe_pos| {
                    maybe_pos.and_then(|pos| calc_tile_height(pos, island_gen, path_cells))
                });

                if maybe_tile_heights[0].is_some() || maybe_tile_heights[1].is_some() {
                    let corner_offsets = [ivec2(-1, -1), ivec2(-1, 1), ivec2(1, 1), ivec2(1, -1)]
                        .map(|offset| {
                            0.5 * vec3(offset.x as f32, 0.0, offset.y as f32) * horizontal_edge_size
                        });

                    let corners = [0, 1, 2, 3].map(|i| {
                        vec3(
                            center_pos.x,
                            maybe_tile_heights[i / 2].unwrap_or(sea_level) as f32,
                            center_pos.y,
                        ) + corner_offsets[i]
                    });

                    let terrain = GrassPath; // TODO

                    heightmap_faces.push(HeightmapMeshFace {
                        geometry: Quad(corners[0], corners[1], corners[2], corners[3]),
                        tile_kind,
                        terrain,
                    });
                }
            } else if z_even {
                let tile_kind = EdgeZ;

                let tile_offsets = [ivec2(0, -1), ivec2(0, 1)];

                let maybe_tile_pos = tile_offsets.map(|offset| {
                    tile_component_pos
                        .checked_add_signed(offset)
                        .and_then(|pos| Some(pos / 2))
                });

                let maybe_tile_heights = maybe_tile_pos.map(|maybe_pos| {
                    maybe_pos.and_then(|pos| calc_tile_height(pos, island_gen, path_cells))
                });

                if maybe_tile_heights[0].is_some() || maybe_tile_heights[1].is_some() {
                    let corner_offsets = [ivec2(1, -1), ivec2(-1, -1), ivec2(-1, 1), ivec2(1, 1)]
                        .map(|offset| {
                            0.5 * vec3(offset.x as f32, 0.0, offset.y as f32) * vertical_edge_size
                        });

                    let corners = [0, 1, 2, 3].map(|i| {
                        vec3(
                            center_pos.x,
                            maybe_tile_heights[i / 2].unwrap_or(sea_level) as f32,
                            center_pos.y,
                        ) + corner_offsets[i]
                    });

                    let terrain = GrassPath; // TODO

                    heightmap_faces.push(HeightmapMeshFace {
                        geometry: Quad(corners[0], corners[1], corners[2], corners[3]),
                        tile_kind,
                        terrain,
                    });
                }
            } else {
                if let Some(y) = calc_tile_height(tile_component_pos / 2, island_gen, path_cells) {
                    let corner_pos = vec3(center_pos.x, y as f32, center_pos.y) - 0.5 * tile_size;

                    info!("{y}");

                    // let terrain = if path_cells.contains_key(&(tile_component_pos / 2)) {
                    //     if y < 22 { // 18, 19
                    //         SandPath
                    //     } else if y < 25 { // 20, 21
                    //         GrassPath
                    //     } else if y < 30 { // 22
                    //         MountainPath
                    //     } else {
                    //         SnowPath
                    //     }
                    //     // GrassPath
                    // } else {
                    //     // Grass
                    //     if y < 22 { // 18, 19
                    //         Sand
                    //     } else if y < 25 { // 20, 21
                    //         Grass
                    //     } else if y < 30 { // 22
                    //         Mountain
                    //     } else {
                    //         Snow
                    //     }
                    // };

                    let terrain = GrassPath; // TODO

                    heightmap_faces.push(HeightmapMeshFace {
                        geometry: Quad(
                            corner_pos,
                            corner_pos + Vec3::Z * tile_size.z,
                            corner_pos + tile_size,
                            corner_pos + Vec3::X * tile_size.x,
                        ),
                        tile_kind: Center,
                        terrain,
                    });
                }
            }
        }
    }

    // TODO: current number is not accurate for this approach
    let num_vertices = 4 * island_size.x * island_size.z;
    let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(num_vertices as usize);
    let mut indices = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut texture_indices: Vec<u32> = Vec::with_capacity(num_vertices as usize);

    for face in heightmap_faces {
        let texture_index = match face.terrain {
            Grass => 0,
            GrassPath => 1,
            DirtSlope => 2,
            StoneSlope => 3,
            Sand => 4,
            SandPath => 5,
            Snow => 6,
            SnowPath => 7,
            Mountain => 8,
            MountainPath => 9,
        };

        match face.geometry {
            Quad(lower_right, upper_right, upper_left, lower_left) => {
                add_quad_geometry(
                    &mut vertices,
                    &mut indices,
                    lower_right,
                    upper_right,
                    upper_left,
                    lower_left,
                );

                // match face.tile_kind {
                //     Center => todo!(),
                //     EdgeX => todo!(),
                //     EdgeZ => todo!(),
                //     Corner => todo!(),
                // }

                texture_indices.push(texture_index);
                texture_indices.push(texture_index);
                texture_indices.push(texture_index);
                texture_indices.push(texture_index);

                let (u, v) = calculate_uv_vectors(lower_right, lower_left, upper_right);
                let centroid = (lower_right + upper_left) * 0.5;
                let uv_origin = centroid;

                let uv_offset = match face.tile_kind {
                    TileKind::Center => Vec2::splat(0.5),
                    TileKind::EdgeX => vec2(0.0, 0.5),
                    TileKind::EdgeZ => vec2(0.5, 0.0),
                    TileKind::Corner => Vec2::ZERO,
                };

                let uv1 = vec2(
                    (lower_right - uv_origin).dot(u),
                    (lower_right - uv_origin).dot(v),
                ) + uv_offset;

                let uv2 = vec2(
                    (upper_right - uv_origin).dot(u),
                    (upper_right - uv_origin).dot(v),
                ) + uv_offset;

                let uv3 = vec2(
                    (upper_left - uv_origin).dot(u),
                    (upper_left - uv_origin).dot(v),
                ) + uv_offset;

                let uv4 = vec2(
                    (lower_left - uv_origin).dot(u),
                    (lower_left - uv_origin).dot(v),
                ) + uv_offset;

                uvs.push(uv1.into());
                uvs.push(uv2.into());
                uvs.push(uv3.into());
                uvs.push(uv4.into());
            }
            Tri(v1, v2, v3) => {
                add_tri_geometry(&mut vertices, &mut indices, v1, v2, v3);

                texture_indices.push(texture_index);
                texture_indices.push(texture_index);
                texture_indices.push(texture_index);

                let (u, v) = calculate_uv_vectors(v1, v2, v3);

                let centroid = (v2 + v3) * 0.5;
                let uv_origin = centroid;

                let uv1 = vec2((v1 - uv_origin).dot(u), (v1 - uv_origin).dot(v));
                let uv2 = vec2((v2 - uv_origin).dot(u), (v2 - uv_origin).dot(v));
                let uv3 = vec2((v3 - uv_origin).dot(u), (v3 - uv_origin).dot(v));

                uvs.push(uv1.into());
                uvs.push(uv2.into());
                uvs.push(uv3.into());
            }
        }
    }

    info!("vertices: {}", vertices.len());
    info!("triangles: {}", indices.len() / 3);

    let mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(
        PixelArtMaterialExtension::ATTRIBUTE_TEXTURE_INDEX,
        texture_indices,
    )
    .with_inserted_indices(bevy::mesh::Indices::U32(indices.clone()))
    .with_computed_normals();

    // let v = vec![
    //     [0.0, 0.0, 0.0],
    //     [0.0, 0.0, 1.0],
    //     [1.0, 0.0, 1.0],
    //     [0.0, 0.0, 0.0],
    //     [1.0, 0.0, 1.0],
    //     [1.0, 0.0, 0.0],
    // ];

    // let i = vec![0, 1, 2, 3, 4, 5];

    // let u = vec![
    //     [0.0, 0.0],
    //     [0.0, 1.0],
    //     [1.0, 1.0],
    //     [1.0, 0.0],
    // ];

    // let t = vec![0u32, 1, 2, 3];

    // let mesh = Mesh::new(
    //     bevy::mesh::PrimitiveTopology::TriangleList,
    //     RenderAssetUsages::default(),
    // )
    // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, v)
    // .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, u)
    // .with_inserted_attribute(
    //     PixelArtMaterialExtension::ATTRIBUTE_TEXTURE_INDEX,
    //     t,
    // )
    // .with_inserted_indices(bevy::mesh::Indices::U32(i.clone()))
    // .with_computed_normals();

    let color_texture_array = asset_server.load_with_settings(
        "textures/terrain_array.png",
        |settings: &mut ImageLoaderSettings| {
            settings.array_layout = Some(ImageArrayLayout::RowHeight { pixels: 32 });
        },
    );

    let material = ExtendedMaterial {
        base: StandardMaterial {
            // base_color: Color::srgb(0.18, 0.35, 0.12),
            metallic: 0.0,
            perceptual_roughness: 0.99,
            reflectance: 0.01,
            // unlit: true,
            // cull_mode: None,
            ..default()
        },
        extension: PixelArtMaterialExtension {
            color_texture_array,
        },
    };

    let scale = vec3(1.0 / tile_size.x, 1.0, 1.0 / tile_size.z);

    commands.spawn((
        Name::new("IslandMesh"),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_scale(scale),
        Wireframe,
        ShowAabbGizmo::default(),
    ));

    let testmesh = meshes.add(Rectangle::new(1.0, 1.0));

    let testmat = testmats.add(StandardMaterial {
        // base_color: Color::srgb(0.18, 0.35, 0.12),
        base_color_texture: Some(asset_server.load("textures/creature_array.png")),
        alpha_mode: AlphaMode::Mask(1.0),
        metallic: 0.0,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        // unlit: true,
        cull_mode: None,
        ..default()
    });

    let testh =
        calc_tile_height(island_gen.center().xz().as_uvec2(), island_gen, path_cells).unwrap();

    let mut testpos = island_gen.center();
    testpos.y = testh as f32;
    testpos += Vec3::splat(0.5);
    testpos *= scale;

    // reference sprite
    commands.spawn((
        Name::new("Shroomington1"),
        Mesh3d(testmesh.clone()),
        MeshMaterial3d(testmat.clone()),
        Transform::from_translation(testpos),
        // Wireframe,
    ));

    commands.spawn((
        Name::new("Shroomington2"),
        Mesh3d(testmesh.clone()),
        MeshMaterial3d(testmat.clone()),
        Transform::from_translation(testpos + vec3(1.0, 0.0, 0.0) * scale),
    ));

    commands.spawn((
        Name::new("Shroomington3"),
        Mesh3d(testmesh),
        MeshMaterial3d(testmat),
        Transform::from_translation(testpos + vec3(-1.0, 1.0, 1.0) * scale),
    ));
}

fn generate_heightmap_mesh_face(tile_component_coords: UVec2) {}

fn calc_tile_height(
    map_offset: UVec2,
    island_gen: &IslandGenerator,
    path_cells: &HashMap<UVec2, PathKind>,
) -> Option<i32> {
    let map_pos = map_offset;
    let mut path_distance: Option<f32> = None;

    // this is pretty inefficient, but fine for now unless island gen gets noticeably slow
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

    let sample_pos = map_pos.as_vec2() + Vec2::splat(0.5);

    let smooth_elevation: f32 = island_gen.smooth_elevation_noise.sample(sample_pos);
    let high_freq_elevation: f32 = island_gen.high_freq_elevation_noise.sample(sample_pos);

    let base_elevation =
        (1.0 - high_freq_weight) * smooth_elevation + (high_freq_weight * high_freq_elevation);

    let island_size = island_gen.full_dimensions();

    let edge_distance = uvec4(
        map_offset.x,
        map_offset.y,
        island_size.x - map_offset.x,
        island_size.z - map_offset.y,
    )
    .min_element();

    let scaled_edge_distance =
        (edge_distance.min(island_gen.boundary_size) as f32) / (island_gen.boundary_size as f32);

    let elevation = base_elevation + scaled_edge_distance.sqrt() - 1.0;

    // TODO don't hardcode
    let sea_level_ratio = 0.6;

    if elevation >= 0.0 {
        let sea_level_offset = (sea_level_ratio * (island_size.y as f32)).round() as u32;
        let sea_level = sea_level_offset as i32;
        let elevation_scale = island_size.y - sea_level_offset;

        let y = sea_level + (elevation * (elevation_scale as f32)).round() as i32;

        Some(y)
    } else {
        None
    }
}

fn add_quad_geometry(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    lower_right: Vec3,
    upper_right: Vec3,
    upper_left: Vec3,
    lower_left: Vec3,
) {
    let index = vertices.len() as u32;

    // only care about relative comparison so distance squared is cheaper
    let diagonal_1_distance = lower_right.distance_squared(upper_left);
    let diagonal_2_distance = lower_left.distance_squared(upper_right);

    vertices.push(lower_right.into());
    vertices.push(upper_right.into());
    vertices.push(upper_left.into());
    vertices.push(lower_left.into());

    if diagonal_1_distance <= diagonal_2_distance {
        indices.push(index);
        indices.push(index + 1);
        indices.push(index + 2);

        indices.push(index);
        indices.push(index + 2);
        indices.push(index + 3);
    } else {
        indices.push(index);
        indices.push(index + 1);
        indices.push(index + 3);

        indices.push(index + 1);
        indices.push(index + 2);
        indices.push(index + 3);
    }
}

fn add_tri_geometry(
    vertices: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
) {
    let index = vertices.len() as u32;

    vertices.push(v1.into());
    vertices.push(v2.into());
    vertices.push(v3.into());

    indices.push(index);
    indices.push(index + 1);
    indices.push(index + 2);
}

fn calculate_uv_vectors(point1: Vec3, point2: Vec3, point3: Vec3) -> (Vec3, Vec3) {
    let edge1 = point2 - point1;
    let edge2 = point3 - point1;
    debug_assert!((edge1.x == 0.0 && edge2.z == 0.0) || (edge1.z == 0.0 && edge2.x == 0.0));

    let normal = edge1.cross(edge2).normalize();
    // debug_assert!(normal.y >= 0.0);

    let d = -normal.x * point1.x - normal.y * point1.y - normal.z * point1.z;

    let maybe_x_intersect = if normal.x != 0.0 {
        Some((-d / normal.x) * Vec3::X)
    } else {
        None
    };
    let maybe_y_intersect = if normal.y != 0.0 {
        Some((-d / normal.y) * Vec3::Y)
    } else {
        None
    };
    let maybe_z_intersect = if normal.z != 0.0 {
        Some((-d / normal.z) * Vec3::Z)
    } else {
        None
    };

    match (maybe_x_intersect, maybe_y_intersect, maybe_z_intersect) {
        (None, None, None) => {
            warn!("degenerate plane");
            (Vec3::X, Vec3::Y)
        }

        (None, None, Some(_)) => (Vec3::X, Vec3::Y),
        (None, Some(_), None) => (Vec3::X, Vec3::Z),
        (Some(_), None, None) => (Vec3::Y, Vec3::Z),

        (None, Some(y_intersect), Some(z_intersect)) => {
            (Vec3::X, (z_intersect - y_intersect).normalize())
        }
        (Some(x_intersect), None, Some(z_intersect)) => {
            ((z_intersect - x_intersect).normalize(), Vec3::Y)
        }
        (Some(x_intersect), Some(y_intersect), None) => {
            ((x_intersect - y_intersect).normalize(), Vec3::Z)
        }

        (Some(x_intersect), Some(y_intersect), Some(z_intersect)) => {
            let u = (x_intersect - z_intersect).normalize();
            let v_edge = z_intersect - y_intersect;
            let v = (v_edge - v_edge.project_onto(u) - v_edge.project_onto(normal)).normalize();

            (u.x.signum() * u, v.y.signum() * -normal.z.signum() * v)
        }
    }
}
