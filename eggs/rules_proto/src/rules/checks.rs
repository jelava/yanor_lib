use bevy::prelude::*;

use crate::rules::*;

#[derive(Component)]
pub(crate) struct Condition<C> {
    pub index: usize,
    pub condition: C,
}

pub(crate) struct Check<const N: usize>;

#[derive(Component)]
pub(crate) struct Index(pub usize);

pub(super) fn new_condition_starting(rule_exec_status: Res<RulesetExecutionStatus>) -> bool {
    rule_exec_status.action_index == 0
}

pub(super) fn check<const N: usize>(
    mut commands: Commands,
    // turn_counter: Res<TurnCounter>,
    mut counter: Local<usize>,
    rule_exec_status: Res<RulesetExecutionStatus>,
    condition_query: Query<(Entity, &Index, &Condition<Check<N>>), With<Energized>>,
) {
    for (entity, Index(i), condition) in &condition_query {
        // TODO: could an entity have ConditionMatched carry over too long (i.e. from previous condition?)
        // might be simpler to just do a per-entity pre-pass before this to remove ConditionMatched and insert FallbackNeeded

        if condition.index == rule_exec_status.condition_index {
            let mut entity_commands = commands.entity(entity);
            entity_commands.insert(FallbackNeeded);

            // info!("{}", *counter % N);
            if *counter % N == 0 {
                // info!("{entity} check {N} passed");
                entity_commands.insert(ConditionMatched);
            } else {
                // info!("{entity} check {N} failed");
                entity_commands.try_remove::<ConditionMatched>();
            }
        }

        *counter += 1;
    }
}
