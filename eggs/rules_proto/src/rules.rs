pub(crate) mod actions;
pub(crate) mod checks;

use bevy::prelude::*;

use {actions::*, checks::*};

pub(super) struct RulesPlugin;

impl Plugin for RulesPlugin {
    fn build(&self, app: &mut App) {
        // TODO: expose checks, actions, maybe more as system sets
        app.init_resource::<RulesetExecutionStatus>()
            .init_resource::<TurnCounter>()
            .add_systems(
                PreUpdate,
                (check::<2>, check::<3>, check::<5>, check::<7>).run_if(new_condition_starting),
            )
            .add_systems(Update, (act::<2>, act::<3>))
            .add_systems(
                PostUpdate,
                (
                    next_rule_step,
                    (fallback::<0>, fallback::<1>).run_if(new_condition_starting),
                    (increment_energy, next_turn)
                        .chain()
                        .run_if(all_out_of_energy),
                    check_apocalypse,
                )
                    .chain(),
            );
    }
}

#[derive(Resource, Default)]
pub(crate) struct RulesetExecutionStatus {
    pub condition_index: usize,
    pub action_index: usize,
}

#[derive(Resource, Default)]
pub(crate) struct TurnCounter(pub(crate) usize);

#[derive(Component)]
pub(crate) struct Actor {
    pub energy: isize,
    pub max_energy: isize,
    pub speed: isize,
}

#[derive(Component)]
pub(crate) struct Energized;

#[derive(Component)]
pub(crate) struct Ruleset {
    pub points: Vec<Vec<usize>>,
}

#[derive(Component)]
pub(crate) struct ConditionMatched;

#[derive(Component)]
pub(crate) struct FallbackNeeded;

fn next_rule_step(
    // mut next_turn_state: ResMut<NextState<TurnState>>,
    mut rule_exec_status: ResMut<RulesetExecutionStatus>,
    ruleset_query: Query<&Ruleset, With<Energized>>,
) {
    // info!("=====");

    let actions_not_finished = ruleset_query
        .iter()
        .any(|ruleset| rule_exec_status.action_index < ruleset.points[0].len() - 1);

    if actions_not_finished {
        rule_exec_status.action_index += 1;
    } else {
        rule_exec_status.action_index = 0;

        // TODO: combine this with original check rather than looping twice
        let ruleset_not_finished = ruleset_query
            .iter()
            .any(|ruleset| rule_exec_status.condition_index < ruleset.points.len() - 1);

        if ruleset_not_finished {
            rule_exec_status.condition_index += 1;
        } else {
            rule_exec_status.condition_index = 0;
        }
    }

    // info!("beginning rule {} action {}", rule_exec_status.condition_index, rule_exec_status.action_index);
}

fn all_out_of_energy(energy_query: Query<&Energized>) -> bool {
    energy_query.is_empty()
}

fn next_turn(
    // mut next_turn_state: ResMut<NextState<TurnState>>,
    mut turn_counter: ResMut<TurnCounter>,
) {
    turn_counter.0 += 1;
    info!("===== begin turn {} =====", turn_counter.0);
    // next_turn_state.set(TurnState::CheckConditions);
}

fn increment_energy(mut commands: Commands, mut actor_query: Query<(Entity, &mut Actor)>) {
    for (entity, mut actor) in &mut actor_query {
        actor.energy = isize::min(actor.energy + actor.speed, actor.max_energy);

        if actor.energy > 0 {
            commands.entity(entity).insert(Energized);
        }
    }
}

fn check_apocalypse(mut app_exit: MessageWriter<AppExit>, ruleset_query: Query<&Ruleset>) {
    if ruleset_query.is_empty() {
        info!("everyone is dead...");
        app_exit.write(AppExit::Success);
    }
}
