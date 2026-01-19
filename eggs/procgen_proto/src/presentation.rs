use bevy::{
    ecs::{
        query::{QueryData, ROQueryItem},
        system::{SystemParam, SystemParamItem}
    },
    prelude::*
};

use crate::HandleProvider;

pub trait IntoPresentableBundle: Component + Sized {
    type MeshProvider: HandleProvider<Mesh, Self> + SystemParam;
    type MaterialProvider: HandleProvider<StandardMaterial, Self> + SystemParam;
    type Data: QueryData;

    fn into_presentable_bundle(
        &self,
        mesh_provider: SystemParamItem<Self::MeshProvider>, //Res<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>, //Res<Self::MaterialProvider>,
        data: ROQueryItem<Self::Data>,
    ) -> impl Bundle;

    fn on_add_presentable(
        trigger: On<Add, Self>,
        mut commands: Commands,
        mesh_provider: SystemParamItem<Self::MeshProvider>, //Res<Self::MeshProvider>,
        material_provider: SystemParamItem<Self::MaterialProvider>, //Res<Self::MaterialProvider>,
        query: Query<(&Self, Self::Data)>,
    ) {
        let target = trigger.event_target();

        if let Ok((presentable, data)) = query.get(target) {
            commands
                .entity(target)
                .insert(presentable.into_presentable_bundle(mesh_provider, material_provider, data));
        } else {
            warn!("Couldn't find Presentable data in query");
        }
    }
}
