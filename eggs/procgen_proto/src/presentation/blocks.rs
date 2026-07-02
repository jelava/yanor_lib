use bevy::{
    ecs::{query::ROQueryItem, system::SystemParamItem},
    prelude::*,
};
use yanor_core::grid::GridPos;

use crate::{BlockHandles, HandleProvider, blocks::*, presentation::IntoPresentableBundle};

pub struct BlockPresentationPlugin;

impl Plugin for BlockPresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(DirtBlock::on_add_presentable)
            .add_observer(GrassBlock::on_add_presentable)
            .add_observer(PathBlock::on_add_presentable)
            .add_observer(StoneBlock::on_add_presentable);
    }
}

impl<B: Block> IntoPresentableBundle for B
where
    BlockHandles: HandleProvider<Mesh, B> + HandleProvider<StandardMaterial, B>,
{
    type MeshProvider = Res<'static, BlockHandles>;
    type MaterialProvider = Res<'static, BlockHandles>;
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

impl HandleProvider<StandardMaterial, DirtBlock> for BlockHandles {
    fn handle(self, _component: &DirtBlock) -> Handle<StandardMaterial> {
        self.dirt_material.clone()
    }
}

impl HandleProvider<StandardMaterial, GrassBlock> for BlockHandles {
    fn handle(self, _component: &GrassBlock) -> Handle<StandardMaterial> {
        self.grass_material.clone()
    }
}

impl HandleProvider<StandardMaterial, PathBlock> for BlockHandles {
    fn handle(self, _component: &PathBlock) -> Handle<StandardMaterial> {
        self.path_material.clone()
    }
}

impl HandleProvider<StandardMaterial, StoneBlock> for BlockHandles {
    fn handle(self, _component: &StoneBlock) -> Handle<StandardMaterial> {
        self.stone_material.clone()
    }
}
