use bevy::{
    ecs::{query::ROQueryItem, system::SystemParamItem},
    prelude::*,
};
use yanor_core::grid::GridPos;

use crate::{BasicHandles, HandleProvider, presentation::IntoPresentableBundle};

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(DirtBlock::on_add_presentable)
            .add_observer(GrassBlock::on_add_presentable)
            .add_observer(PathBlock::on_add_presentable)
            .add_observer(StoneBlock::on_add_presentable);
    }
}

pub trait Block: Component {}

#[derive(Component)]
pub struct DirtBlock;
impl Block for DirtBlock {}

#[derive(Component)]
pub struct GrassBlock;
impl Block for GrassBlock {}

#[derive(Component)]
pub struct PathBlock;
impl Block for PathBlock {}

#[derive(Component)]
pub struct StoneBlock;
impl Block for StoneBlock {}

impl<B: Block> IntoPresentableBundle for B
where
    BasicHandles: HandleProvider<Mesh, B> + HandleProvider<StandardMaterial, B>,
{
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
