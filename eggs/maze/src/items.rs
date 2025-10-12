pub mod activities;

use bevy::prelude::*;
use yanor_core::{activity::*, grid::GridPosition};

use activities::*;

pub use activities::{
    DropItem, GRAB_ITEM_DURATION_STAT_ID, GetItem, PLACE_ITEM_DURATION_STAT_ID,
    STORE_ITEM_DURATION_STAT_ID, TAKE_OUT_ITEM_DURATION_STAT_ID,
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_activity::<GetItem>()
            .init_activity::<DropItem>()
            .add_observer(on_item_add_stored_in)
            .add_observer(on_item_remove_stored_in)
            .add_observer(on_inventory_despawn)
            .add_observer(on_get_item_phase_finished)
            .add_observer(on_drop_item_phase_finished);
    }
}

/// This component indicates that an entity can be stored by entities that have the Inventory
/// component.
#[derive(Component, Default)]
pub struct Item;

/// This component indicates that an entity can store entities with the Item component.
#[derive(Component, Default)]
pub struct Inventory {
    pub capacity: Option<u32>,
    item_count: u32,
}

/// Relationship between an entity with Item and the entity with Inventory that currently stores it.
#[derive(Component)]
#[relationship(relationship_target = Stores)]
pub struct StoredIn(pub Entity);

/// Target of the StoredIn relationship, so it contains all the item stored in a given inventory.
#[derive(Component)]
#[relationship_target(relationship = StoredIn)]
pub struct Stores(Vec<Entity>);

fn on_item_add_stored_in(
    trigger: On<Add, StoredIn>,
    mut commands: Commands,
    stored_in_query: Query<&StoredIn>,
    mut inventory_query: Query<&mut Inventory>,
) {
    let target = trigger.event_target();

    if let Ok(&StoredIn(inventory_entity)) = stored_in_query.get(target) {
        if let Ok(mut inventory) = inventory_query.get_mut(inventory_entity) {
            if inventory
                .capacity
                .is_some_and(|capacity| inventory.item_count >= capacity - 1)
            {
                warn!("Tried to add item to already full inventory");

                // Remove the relationship since the inventory is full and the item can't be stored
                // in it
                commands.entity(target).remove::<StoredIn>();
            } else {
                inventory.item_count += 1;

                // TODO? should this even be removed? maybe it should just be updated in sync w/ grid pos of
                // inventory entity probably cheaper/simpler to just sync the grid pos only once the item is
                // removed from inventory
                commands
                    .entity(target)
                    .remove::<GridPosition>()
                    .insert(Visibility::Hidden);
            }
        } else {
            warn!("Can't find inventory component on target of StoredIn relationship");
        }
    } else {
        warn!("Can't find entity that {target:?} is being stored in");
    }
}

fn on_item_remove_stored_in(
    trigger: On<Remove, StoredIn>,
    mut commands: Commands,
    stored_in_query: Query<&StoredIn>,
    mut inventory_query: Query<(&mut Inventory, &GridPosition)>,
) {
    let target = trigger.event_target();

    if let Ok(&StoredIn(inventory_entity)) = stored_in_query.get(target) {
        if let Ok((mut inventory, grid_pos)) = inventory_query.get_mut(inventory_entity) {
            if inventory.item_count > 0 {
                inventory.item_count -= 1;
            } else {
                warn!("Removed item from inventory that already has item_count of 0");
            }

            commands
                .entity(target)
                .insert(*grid_pos)
                .insert(Visibility::Inherited);
        } else {
            warn!(
                "Can't find Inventory and/or GridPosition component on target of StoredIn relationship"
            );
        }
    } else {
        warn!("Can't find entity that {target:?} is being stored in");
    }
}

fn on_inventory_despawn(trigger: On<Despawn, Inventory>) {
    todo!("Dump out all the items in the inventory at the location where it despawned");
}
