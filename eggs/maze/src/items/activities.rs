use bevy::prelude::*;
use yanor_core::{activity::*, stats::StatId};

use crate::items::*;

pub struct GetItem(pub Entity);

impl Activity for GetItem {
    type Phase = GetItemPhase;

    fn name(&self) -> String {
        "Get item".into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([GetItemPhase::PickUpItem, GetItemPhase::StoreItem].into())
    }
}

// TODO: this should probably be broken up into two separate activities
// One called GrabItem for picking something up and holding it in a hand/other suitable equipment slot
// One called StoreItem for taking something in an equipment slot
// Then, GetItem can be a "compound activity" composed of GrabItem and StoreItem
#[derive(Clone, Copy)]
pub enum GetItemPhase {
    PickUpItem,
    StoreItem,
}

// TODO! this is terrible lol
pub const GRAB_ITEM_DURATION_STAT_ID: StatId = StatId(1);
pub const STORE_ITEM_DURATION_STAT_ID: StatId = StatId(2);

impl ActivityPhase for GetItemPhase {
    fn name(&self) -> String {
        use GetItemPhase::*;

        match self {
            PickUpItem => "Pick up item".into(),
            StoreItem => "Store item".into(),
        }
    }

    fn duration(&self) -> StatId {
        use GetItemPhase::*;

        match self {
            PickUpItem => GRAB_ITEM_DURATION_STAT_ID,
            StoreItem => STORE_ITEM_DURATION_STAT_ID,
        }
    }
}

pub(super) fn on_get_item_phase_finished(
    trigger: On<FinishActivityPhase<GetItemPhase>>,
    mut commands: Commands,
    activity_query: Query<&Active<GetItem>, With<Inventory>>,
) {
    use GetItemPhase::*;

    let target = trigger.event_target();

    if let Ok(&Active(GetItem(item_entity))) = activity_query.get(target) {
        match trigger.event().phase {
            PickUpItem => {
                warn!("TODO: equip/hold the item in the player's hand")
            }
            StoreItem => {
                commands.entity(item_entity).insert(StoredIn(target));
            }
        }
    } else {
        warn!("Target of GetItemPhase finished trigger not found in query");
    }
}

pub struct DropItem(pub Entity);

impl Activity for DropItem {
    type Phase = DropItemPhase;

    fn name(&self) -> String {
        todo!()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue<Self::Phase> {
        ActivityPhaseQueue::new([DropItemPhase::TakeOutItem, DropItemPhase::PlaceItem].into())
    }
}

pub const TAKE_OUT_ITEM_DURATION_STAT_ID: StatId = StatId(3);
pub const PLACE_ITEM_DURATION_STAT_ID: StatId = StatId(4);

#[derive(Clone, Copy)]
pub enum DropItemPhase {
    TakeOutItem,
    PlaceItem,
}

impl ActivityPhase for DropItemPhase {
    fn name(&self) -> String {
        todo!()
    }

    fn duration(&self) -> StatId {
        use DropItemPhase::*;

        match self {
            TakeOutItem => TAKE_OUT_ITEM_DURATION_STAT_ID,
            PlaceItem => PLACE_ITEM_DURATION_STAT_ID,
        }
    }
}

pub(super) fn on_drop_item_phase_finished(
    trigger: On<FinishActivityPhase<DropItemPhase>>,
    mut commands: Commands,
    activity_query: Query<&Active<DropItem>, With<Inventory>>,
) {
    use DropItemPhase::*;

    let target = trigger.event_target();

    if let Ok(&Active(DropItem(item_entity))) = activity_query.get(target) {
        match trigger.event().phase {
            TakeOutItem => {
                warn!("TODO: equip/hold the item in the player's hand")
            }
            PlaceItem => {
                commands.entity(item_entity).remove::<StoredIn>();
            }
        }
    }
}
