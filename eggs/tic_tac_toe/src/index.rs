/// Traits for Resources that help with looking up entities by component value
use std::{hash::Hash, marker::PhantomData};

use bevy::{
    ecs::entity::EntityHashSet,
    platform::collections::HashMap,
    prelude::*,
};

pub trait ComponentIndex: Resource {
    type Cmp: Component + Copy + Clone;

    fn get(&self, component: &Self::Cmp) -> Option<&EntityHashSet>;
    fn insert(&mut self, component: Self::Cmp, entity: Entity);
    fn remove(&mut self, component: &Self::Cmp, entity: Entity);
}

/// Useful for keeping track of locations of things that are scattered across a wide area with no extra
/// memory overhead, or which don't need to be efficiently accessible in sequence (an underlying data
/// structure with better spatial locality will do better for that).
#[derive(Resource)]
pub struct SparseComponentIndex<C: Component + Eq + Hash> {
    data: HashMap<C, EntityHashSet>,
}

// Manually implement rather than derive to prevent complaints about C not implementing Default
impl<C: Component + Eq + Hash> Default for SparseComponentIndex<C> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<C: Component + Copy + Clone + Eq + Hash> ComponentIndex for SparseComponentIndex<C> {
    type Cmp = C;

    fn get(&self, component: &C) -> Option<&EntityHashSet> {
        self.data.get(component)
    }

    fn insert(&mut self, component: C, entity: Entity) {
        if let Some(entity_set) = self.data.get_mut(&component) {
            entity_set.insert(entity);
        } else {
            self.data.insert(component, EntityHashSet::from([entity]));
        }
    }

    fn remove(&mut self, component: &C, entity: Entity) {
        if let Some(entity_set) = self.data.get_mut(component) {
            entity_set.remove(&entity);

            if entity_set.is_empty() {
                self.data.remove(component);
            }
        }
    }
}

/// Generic plugin for setting up a ComponentIndex. Handles initializing the index as a resource
/// and registering the component hooks to maintain the index.
pub struct ComponentIndexPlugin<I: ComponentIndex + Default>(PhantomData<I>);

impl<I: ComponentIndex + Default> Default for ComponentIndexPlugin<I> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<I: ComponentIndex + Default> Plugin for ComponentIndexPlugin<I> {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<I>()
            .add_observer(update_index_on_insert::<I>)
            .add_observer(update_index_on_replace::<I>);
            //.add_systems(PreStartup, register_component_index_hooks::<I>);
    }
}

// fn register_component_index_hooks<I: ComponentIndex>(world: &mut World) {
//     // TODO: this will panic if I::Cmp already has insert/replace hooks
//     // (maybe just use observers? pros/cons to switching?)
//     world
//         .register_component_hooks::<I::Cmp>()
//         .on_insert(|mut world, HookContext { entity, .. }| {
//             let component = *world.get(entity).unwrap();
//             world.resource_mut::<I>().insert(component, entity);
//         })
//         .on_replace(|mut world, HookContext { entity, .. }| {
//             let component = *world.get(entity).unwrap();
//             world.resource_mut::<I>().remove(&component, entity);
//
//             // The insert hook is guaranteed to run after this if the component is being replaced
//             // and it will handle re-adding the entity to the index for the new component value
//         });
// }

fn update_index_on_insert<I: ComponentIndex>(
    trigger: Trigger<OnInsert, I::Cmp>,
    mut index: ResMut<I>,
    component_query: Query<&I::Cmp>,
) {
    let entity = trigger.target();

    if let Ok(&component) = component_query.get(entity) {
        index.insert(component, entity);
    } else {
        panic!("Entity does not have indexed component");
    }
}

fn update_index_on_replace<I: ComponentIndex>(
    trigger: Trigger<OnReplace, I::Cmp>,
    mut index: ResMut<I>,
    component_query: Query<&I::Cmp>,
) {
    let entity = trigger.target();

    if let Ok(component) = component_query.get(entity) {
        index.remove(component, entity);
    } else {
        panic!("Entity does not have indexed component");
    }
}
