use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use yanor_core::{activity::*, grid::*, stats::*};

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.init_activity::<OpenDoor>()
            .init_activity::<CloseDoor>()
            .add_observer(on_open_door_phase_finish)
            .add_observer(on_close_door_phase_finish);
    }
}

// fn insert_hook<B: Bundle + Default>(
//     mut world: DeferredWorld,
//     HookContext { entity, .. }: HookContext,
// ) {
//     info!("inserting");

//     world
//         .commands()
//         .entity(entity)
//         .insert(B::default());
// }

fn remove_hook<B: Bundle>(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    info!("removing");

    world.commands().entity(entity).try_remove::<B>();
}

#[derive(Component, Default)]
#[component(
    storage = "SparseSet",
    on_insert = remove_hook::<Closed>,
    // on_remove = insert_hook::<Closed>,
)]
pub struct Open;

#[derive(Component, Default)]
#[component(
    storage = "SparseSet",
    on_insert = remove_hook::<Open>,
    // on_remove = insert_hook::<Open>,
)]
pub struct Closed;

#[derive(Component, Default, PartialEq, Eq)]
#[component(immutable)]
pub enum DoorOrientation {
    #[default]
    FacingZ,
    FacingX,
}

#[derive(Component)]
#[require(DoorOrientation, GridPos)]
pub struct Door;

pub struct OpenDoor(pub Entity);

impl Activity for OpenDoor {
    type Phase = OpenDoorPhase;

    fn name(&self) -> String {
        todo!()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([OpenDoorPhase].into())
    }
}

pub const OPEN_DOOR_DURATION_STAT_ID: StatId = StatId(5);

#[derive(Clone, Copy)]
pub struct OpenDoorPhase;

impl ActivityPhase for OpenDoorPhase {
    fn name(&self) -> String {
        todo!()
    }

    fn duration(&self) -> yanor_core::stats::StatId {
        OPEN_DOOR_DURATION_STAT_ID
    }
}

fn on_open_door_phase_finish(
    trigger: On<FinishActivityPhase<OpenDoorPhase>>,
    mut commands: Commands,
    activity_query: Query<(&Active<OpenDoor>, &GridPos)>,
    door_query: Query<(&GridPos, &DoorOrientation), With<Door>>,
) {
    let target = trigger.event_target();

    info!("Try to open door");

    if let Ok((&Active(OpenDoor(door_entity)), &GridPos(active_pos))) = activity_query.get(target) {
        if let Ok((&GridPos(door_pos), door_orientation)) = door_query.get(door_entity) {
            if check_door_adjacency(active_pos, door_pos, door_orientation) {
                info!("insert open");

                commands.entity(door_entity).insert(Open);
            }
        } else {
            warn!("Couldn't find GridPosition of Door");
        }
    } else {
        warn!("Couldn't find target of OpenDoor activity");
    }
}

pub struct CloseDoor(pub Entity);

impl Activity for CloseDoor {
    type Phase = CloseDoorPhase;

    fn name(&self) -> String {
        todo!()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([CloseDoorPhase].into())
    }
}

pub const CLOSE_DOOR_DURATION_STAT_ID: StatId = StatId(6);

#[derive(Clone, Copy)]
pub struct CloseDoorPhase;

impl ActivityPhase for CloseDoorPhase {
    fn name(&self) -> String {
        todo!()
    }

    fn duration(&self) -> StatId {
        CLOSE_DOOR_DURATION_STAT_ID
    }
}

fn on_close_door_phase_finish(
    trigger: On<FinishActivityPhase<CloseDoorPhase>>,
    mut commands: Commands,
    activity_query: Query<(&Active<CloseDoor>, &GridPos)>,
    door_query: Query<(&GridPos, &DoorOrientation), With<Door>>,
) {
    let target = trigger.event_target();

    info!("Try to close door");

    if let Ok((&Active(CloseDoor(door_entity)), &GridPos(active_pos))) = activity_query.get(target)
    {
        if let Ok((&GridPos(door_pos), door_orientation)) = door_query.get(door_entity) {
            if check_door_adjacency(active_pos, door_pos, door_orientation) {
                commands.entity(door_entity).insert(Closed);
            }
        } else {
            warn!("Couldn't find GridPosition of Door");
        }
    } else {
        warn!("Couldn't find target of CloseDoor activity");
    }
}

fn check_door_adjacency(
    active_pos: IVec3,
    door_pos: IVec3,
    door_orientation: &DoorOrientation,
) -> bool {
    use DoorOrientation::*;

    let door_offset = (door_pos - active_pos).abs();
    info!("{door_offset:?}");

    match door_orientation {
        &FacingX => door_offset == IVec3::X,
        &FacingZ => door_offset == IVec3::Z,
    }
}
