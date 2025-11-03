use bevy::prelude::*;
use yanor_core::{
    activity::*, grid::*, index::ComponentIndex, input::ActiveInputController, tick::*,
};

use crate::{StepLatencyStopwatch, door::*, items::*, step::*};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                process_movement_inputs.run_if(in_state(TickState::PreTick)),
                process_item_inputs.run_if(in_state(TickState::PreTick)),
                process_door_inputs.run_if(in_state(TickState::PreTick)),
            ),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Name(String);

fn process_movement_inputs(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid_index: Res<SparseGridIndex>,
    active_input_controller_query: Single<
        (Entity, &GridPosition),
        (With<ActiveInputController>, With<Inactive>),
    >,
    mut step_latency_stopwatch: ResMut<StepLatencyStopwatch>,
) {
    use AxisDirection::*;

    let (entity, &pos) = *active_input_controller_query;
    let mut entity_commands = commands.entity(entity);

    let direction;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        direction = GridDirection::new(Zero, Zero, Plus);
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        direction = GridDirection::new(Zero, Zero, Minus);
    } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        direction = GridDirection::new(Plus, Zero, Zero);
    } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        direction = GridDirection::new(Minus, Zero, Zero);
    } else {
        return;
    }

    // if !grid_index.contains(&(pos + direction)) { // TODO: also check if there is something below to stand on!
    entity_commands.insert(Active(Step(direction)));
    // } else {
    // todo!("give some kind of feedback indicating that the player can't move there");
    // }

    step_latency_stopwatch.0.unpause();
}

// TODO: break this up into a multi-stage input (i.e. 1st input = choose activity, 2nd input = choose which item to get/drop)
fn process_item_inputs(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid_index: Res<SparseGridIndex>,
    active_controller_query: Single<
        (Entity, &GridPosition, Option<&Stores>),
        (With<ActiveInputController>, With<Inactive>, With<Inventory>),
    >,
    stored_items_query: Query<Entity, (With<Item>, With<StoredIn>)>,
    unstored_items_query: Query<Entity, (With<Item>, Without<StoredIn>)>,
) {
    let (input_entity, input_pos, maybe_stores) = *active_controller_query;

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        let mut selected_item = None;

        if let Some(entity_set) = grid_index.get(input_pos) {
            for entity in entity_set {
                if let Ok(item) = unstored_items_query.get(*entity) {
                    warn!(
                        "TODO: shouldn't just pick up the first item, need to give player a choice in case of multiple items"
                    );

                    selected_item = Some(item);
                    break;
                }
            }
        }

        if let Some(item) = selected_item {
            info!("storing {item:?} in inventory of {input_entity:?}");

            commands.entity(input_entity).insert(Active(GetItem(item)));
        } else {
            info!("Looks like there are no items here");
        }
    } else if keyboard_input.just_pressed(KeyCode::KeyD) {
        warn!("TODO: select which item to drop (if more than 1 in inventory)");

        if let Some(item) = stored_items_query.iter().next() {
            info!("dropping {item:?} from inventory of {input_entity:?}");

            commands.entity(input_entity).insert(Active(DropItem(item)));
        } else {
            info!("Your inventory is already empty.");
        }
    } else if keyboard_input.just_pressed(KeyCode::KeyI) {
        warn!("TODO: this should display an actual panel/window in the UI");

        if let Some(stores) = maybe_stores {
            let mut item_count = 0;

            for item in stores.collection() {
                item_count += 1;
                info!("item {item_count:?}: {item:?}");
            }
        } else {
            info!("Your inventory is currently empty.");
        }
    }
}

// TODO: create a more flexible/generalized/context-sensitive way of interacting w/ stuff
fn process_door_inputs(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid_index: Res<SparseGridIndex>,
    active_input_controller_pos: Single<
        (Entity, &GridPosition),
        (With<ActiveInputController>, With<Inactive>),
    >,
    door_query: Query<(&DoorOrientation, Has<Open>), With<Door>>
) {
    use DoorOrientation::*;

    let (active_entity, &active_pos) = *active_input_controller_pos;

    if keyboard_input.just_pressed(KeyCode::KeyO) {
        // TODO: this is an extremely lazy and verbose approach and doesn't give control over which door is opened

        if let Some(entities) = grid_index.get(&(active_pos + GridDirection::X)) {
            for &entity in entities {
                if let Ok((door_orientation, door_open)) = door_query.get(entity) {
                    if door_orientation == &FacingX {
                        if door_open {
                            commands
                                .entity(active_entity)
                                .insert(Active(CloseDoor(entity)));
                        } else {
                            commands
                                .entity(active_entity)
                                .insert(Active(OpenDoor(entity)));
                        }

                        return;
                    }
                }
            }
        }

        if let Some(entities) = grid_index.get(&(active_pos + GridDirection::NEG_X)) {
            for &entity in entities {
                if let Ok((door_orientation, door_open)) = door_query.get(entity) {
                    if door_orientation == &FacingX {
                        if door_open {
                            commands
                                .entity(active_entity)
                                .insert(Active(CloseDoor(entity)));
                        } else {
                            commands
                                .entity(active_entity)
                                .insert(Active(OpenDoor(entity)));
                        }

                        return;
                    }
                }
            }
        }

        if let Some(entities) = grid_index.get(&(active_pos + GridDirection::Z)) {
            for &entity in entities {
                if let Ok((door_orientation, door_open)) = door_query.get(entity) {
                    if door_orientation == &FacingZ {
                        if door_open {
                            commands
                                .entity(active_entity)
                                .insert(Active(CloseDoor(entity)));
                        } else {
                            commands
                                .entity(active_entity)
                                .insert(Active(OpenDoor(entity)));
                        }

                        return;
                    }
                }
            }
        }

        if let Some(entities) = grid_index.get(&(active_pos + GridDirection::NEG_Z)) {
            for &entity in entities {
                if let Ok((door_orientation, door_open)) = door_query.get(entity) {
                    if door_orientation == &FacingZ {
                        if door_open {
                            commands
                                .entity(active_entity)
                                .insert(Active(CloseDoor(entity)));
                        } else {
                            commands
                                .entity(active_entity)
                                .insert(Active(OpenDoor(entity)));
                        }

                        return;
                    }
                }
            }
        }

        info!("Couldn't find a door to open/close?");
    }
}
