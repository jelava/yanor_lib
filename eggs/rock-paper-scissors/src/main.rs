mod tick;

use bevy::{log::LogPlugin, prelude::*};

use tick::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default(), TickPlugin))
        .add_systems(Startup, spawn_players)
        .add_systems(
            FixedUpdate,
            (
                process_idle_rock_controllers.run_if(in_state(TickState::PreTick)),
                do_rps_activities.run_if(in_state(TickState::Tick)),
            )
        )
        .run();
}

// Basic stuff

#[derive(Component, Debug)]
struct PlayerId(u32);

// Trivially simple - always chooses rock
#[derive(Component)]
struct RockController;

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub struct PendingTick;

#[derive(Component, Debug)]
enum RpsActivity {
    Rock,
    Paper,
    Scissors,
}

// struct Activity {
//     name: String,
//     phases: Vec<ActivityPhase>
// }
//
// struct ActivityPhase {
//     name: String,
//     duration: usize,
// }
//
// struct CurrentActivity {
//     activity: Activity,
//     phase_index: usize,
//     ticks_remaining: usize,
// }

#[derive(Component, Default)]
struct ScoreTracker {
    wins: usize,
    ties: usize,
    losses: usize,
}

fn spawn_players(mut commands: Commands) {
    info!("spawning 2 NPC players");

    commands.spawn((PlayerId(0), ScoreTracker::default(), RockController, Idle));
    commands.spawn((PlayerId(1), ScoreTracker::default(), RockController, Idle));
}

// Controller/activity stuff

fn process_idle_rock_controllers(
    mut commands: Commands,
    rock_controller_query: Query<Entity, (With<RockController>, With<Idle>)>,
) {
    for entity in &rock_controller_query {
        commands
            .entity(entity)
            .remove::<Idle>()
            .insert((PendingTick, RpsActivity::Rock));
    }
}

enum RpsOutcome {
    Tie,
    Player1Wins,
    Player2Wins,
}

fn do_rps_activities(
    mut commands: Commands,
    mut rps_query: Query<(Entity, &PlayerId, Has<PendingTick>, &RpsActivity, &mut ScoreTracker)>
) {
    use {RpsActivity::*, RpsOutcome::*};

    let mut iter = rps_query.iter_combinations_mut();

    while let Some(
        [
            (entity1, player1, still_pending1, activity1, mut score1),
            (entity2, player2, still_pending2, activity2, mut score2),
        ],
    ) = iter.fetch_next()
    {
        let player_description = format!("Player {player1:?} v. player {player2:?}");

        let (move_description, outcome) = match (activity1, activity2) {
            (Rock, Rock) | (Paper, Paper) | (Scissors, Scissors) => {
                (format!("both chose {activity1:?}"), Tie)
            }
            (Rock, Paper) => ("rock covered by paper".into(), Player2Wins),
            (Rock, Scissors) => ("rock breaks scissors".into(), Player1Wins),
            (Paper, Rock) => ("paper covers rock".into(), Player1Wins),
            (Paper, Scissors) => ("paper cut by scissors".into(), Player2Wins),
            (Scissors, Rock) => ("scissors broken by rock".into(), Player2Wins),
            (Scissors, Paper) => ("scissors cut paper".into(), Player1Wins),
        };

        let outcome_description;

        outcome_description = match outcome {
            Tie => {
                score1.ties += 1;
                score2.ties += 1;
                "Tie".into()
            }
            Player1Wins => {
                score1.wins += 1;
                score2.losses += 1;
                format!("Player {player1:?} wins")
            }
            Player2Wins => {
                score1.losses += 1;
                score2.wins += 1;
                format!("Player {player2:?} wins")
            }
        };

        info!("{player_description}: {move_description}. {outcome_description}!");

        if still_pending1 {
            commands.entity(entity1).remove::<PendingTick>();
        }

        if still_pending2 {
            commands.entity(entity2).remove::<PendingTick>();
        }
    }

    info!("===");
}
