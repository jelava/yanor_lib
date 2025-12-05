use bevy::prelude::*;

use crate::rules::*;

#[derive(Component)]
pub(crate) struct Action<A> {
    pub index: usize,
    pub action: A,
}

#[derive(Component)]
pub(crate) struct Fallback<A> {
    pub action: A,
}

pub(crate) struct Do<const N: usize>;

pub(super) fn act<const N: usize>(
    mut commands: Commands,
    rule_exec_status: Res<RulesetExecutionStatus>,
    mut action_query: Query<
        (Entity, &Action<Do<N>>, &mut Ruleset, &mut Actor),
        (With<Energized>, With<ConditionMatched>),
    >,
) {
    let RulesetExecutionStatus {
        condition_index,
        action_index,
    } = *rule_exec_status;

    for (entity, action, mut ruleset, mut actor) in &mut action_query {
        if action.index == rule_exec_status.action_index
            && ruleset.points[condition_index][action_index] > 0
        {
            commands.entity(entity).try_remove::<FallbackNeeded>();
            // info!("{entity} do action {N}");

            actor.energy -= 100; // TODO: non-hardcoded energy cost

            if actor.energy <= 0 {
                // info!("{entity} out of energy");
                commands.entity(entity).remove::<Energized>();
            }

            ruleset.points[condition_index][action_index] -= 1;

            let total_points: usize = ruleset.points.iter().flatten().sum();

            // info!("{entity} has {total_points} points left");

            if total_points == 0 {
                info!("{entity} out of ruleset points");
                commands.entity(entity).despawn();
            }
        }
    }
}

pub(super) fn fallback<const N: usize>(
    mut commands: Commands,
    mut fallback_query: Query<
        (Entity, &Fallback<Do<N>>, &mut Actor),
        (With<Energized>, With<FallbackNeeded>),
    >,
) {
    for (entity, _fallback, mut actor) in &mut fallback_query {
        // info!("{entity} do fallback {N}");

        actor.energy -= 25; // TODO: non-hardcoded energy cost

        if actor.energy <= 0 {
            // info!("{entity} out of energy");
            commands.entity(entity).remove::<Energized>();
        }
    }
}
