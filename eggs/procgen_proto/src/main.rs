use std::cmp::{max, min};

use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin, ecs::query::{QueryData, ROQueryItem}, prelude::*
};
use bevy_rand::prelude::*;
use yanor_core::grid::GridPos;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            FpsOverlayPlugin::default(),
            EntropyPlugin::<WyRand>::default(),
        ))
        .add_systems(Startup, init)
        .add_observer(Terrain::on_add_presentable)
        .run();
}

#[derive(Component)]
struct IslandGenerator {
    origin: IVec3,
    dimensions: IVec3,
    min_regions: u32,
    max_regions: u32,
}

fn init(mut commands: Commands) {
    commands.spawn(IslandGenerator {
        origin: IVec3::ZERO,
        dimensions: ivec3(10, 10, 10),
        min_regions: 3,
        max_regions: 6,
    });
}

fn generate_islands(mut commands: Commands, island_gen_query: Query<&IslandGenerator>) {
    for island_gen in &island_gen_query {
        let corner = island_gen.origin + island_gen.dimensions;
        let surface_level = (2 * corner.y) / 3;

        for x in island_gen.origin.x..corner.x {
            for z in island_gen.origin.z..corner.z {
                let elevation = max(x, z);
                let surface_height = min(surface_level + elevation, corner.z);

                for y in surface_level..surface_height {
                    commands.spawn((
                        Terrain::Dirt,
                        GridPos::new(x, y, z),
                    ));
                }
            }
        }
    }
}

trait IntoPresentableBundle: Component + Sized {
    type Handles: Resource;
    type Data: QueryData;

    fn into_presentable_bundle(
        &self,
        handles: Res<Self::Handles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle;

    fn on_add_presentable(
        trigger: On<Add, Self>,
        mut commands: Commands,
        handles: Res<Self::Handles>,
        query: Query<(&Self, Self::Data)>,
    ) {
        let target = trigger.event_target();

        if let Ok((presentable, data)) = query.get(target) {
            commands
                .entity(target)
                .insert(presentable.into_presentable_bundle(handles, data));
        } else {
            warn!("Couldn't find Presentable data in query");
        }
    }
}

#[derive(Component)]
struct GrassBlock;

#[derive(Resource)]
struct TerrainHandles {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

impl IntoPresentableBundle for Terrain {
    type Handles = TerrainHandles;
    type Data = &'static GridPos;

    fn into_presentable_bundle(
        &self,
        handles: Res<Self::Handles>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle {
        let grid_pos = *data;

        (
            Transform::from_translation(grid_pos.into()),
            Mesh3d(handles.mesh.clone()),
            MeshMaterial3d(handles.material.clone()),
        )
    }
}
