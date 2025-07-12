mod activity;
mod tick;

use bevy::{log::LogPlugin, prelude::*};

use crate::{activity::*, tick::*};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default(), TickPlugin))
        .insert_resource(RoundCounter {
            current_round: 0,
            num_rounds: 5,
        })
        .add_systems(Startup, spawn_players)
        .add_systems(PostStartup, start_ticking)
        .add_systems(
            FixedUpdate,
            (
                process_idle_rock_controllers.run_if(in_state(TickState::PreTick)),
                do_rps_activities.run_if(in_state(TickState::Tick)),
            ),
        )
        .add_systems(OnExit(TickState::PostTick), increment_round_counter)
        .add_systems(OnEnter(TickState::PreTick), player_status_check)
        .add_systems(OnEnter(TickState::Tick), player_status_check)
        .add_systems(OnEnter(TickState::PostTick), player_status_check)
        .run();
}

// Basic stuff

#[derive(Component, Debug)]
#[require(ScoreTracker)]
struct PlayerId(u32);

#[derive(Component, Default)]
struct ScoreTracker {
    wins: usize,
    ties: usize,
    losses: usize,
}

#[derive(Resource)]
struct RoundCounter {
    current_round: usize,
    num_rounds: usize,
}

fn spawn_players(mut commands: Commands) {
    info!("spawning 2 NPC players");

    commands.spawn((PlayerId(0), RockController));
    commands.spawn((PlayerId(1), RockController));
}

fn increment_round_counter(
    mut round_counter: ResMut<RoundCounter>,
    mut app_exit: EventWriter<AppExit>,
    score_query: Query<(&PlayerId, &ScoreTracker)>,
) {
    info!("===== round {} over! =====", round_counter.current_round);

    round_counter.current_round += 1;

    if round_counter.current_round >= round_counter.num_rounds {
        info!(
            "{} rounds finished, game is finished.",
            round_counter.current_round
        );
        app_exit.write(AppExit::Success);

        for (player, score) in &score_query {
            info!(
                "{player:?} final score: {} wins, {} ties, {} losses. Net total: {}",
                score.wins,
                score.ties,
                score.losses,
                score.wins - score.losses
            );
        }
    }
}

fn player_status_check(
    player_query: Query<(
        &PlayerId,
        Has<Idle>,
        Has<NeedsTick>,
        Has<CurrentActivity<RpsActivity>>,
        Option<&ActivityPhaseQueue>,
    )>,
) {
    for (player, has_idle, has_pending_tick, has_activity, maybe_phase_queue) in &player_query {
        info!(
            "{player:?} (idle: {has_idle}, pending tick: {has_pending_tick}, current_activity: {has_activity})"
        );

        if let Some(phase_queue) = maybe_phase_queue {
            info!(" ^ has phase queue");
        } else {
            info!(" ^ no phase queue");
        }
    }
}

// Controller stuff

// Trivially simple - always chooses rock
#[derive(Component)]
#[require(Idle)]
struct RockController;

fn process_idle_rock_controllers(
    mut commands: Commands,
    rock_controller_query: Query<(Entity, &PlayerId), (With<RockController>, With<Idle>)>,
) {
    for (entity, player) in &rock_controller_query {
        info!("{player:?} is idle, adding Rock as current activity");

        commands
            .entity(entity)
            .insert(CurrentActivity(RpsActivity::Rock));
    }
}

// RPS specific activity stuff

#[derive(Debug)]
enum RpsActivity {
    Rock,
    Paper,
    Scissors,
}

impl Activity for RpsActivity {
    fn name(&self) -> String {
        use RpsActivity::*;

        match self {
            Rock => "rock",
            Paper => "paper",
            Scissors => "scissors",
        }
        .into()
    }

    fn phase_queue(&self) -> ActivityPhaseQueue {
        ActivityPhaseQueue::new(
            [ActivityPhase {
                name: "whatever".into(),
                duration: 1,
            }]
            .into(),
        )
    }
}

enum RpsOutcome {
    Tie,
    Player1Wins,
    Player2Wins,
}

fn do_rps_activities(
    mut commands: Commands,
    mut rps_query: Query<
        (
            Entity,
            &PlayerId,
            &CurrentActivity<RpsActivity>,
            &mut ScoreTracker,
        ),
        With<NeedsTick>,
    >,
) {
    use {RpsActivity::*, RpsOutcome::*};

    let mut iter = rps_query.iter_combinations_mut();

    while let Some(
        [
            (entity1, player1, CurrentActivity(activity1), mut score1),
            (entity2, player2, CurrentActivity(activity2), mut score2),
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

        let outcome_description = match outcome {
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

        commands.entity(entity1).insert(TickDone);
        commands.entity(entity2).insert(TickDone);
    }

    info!("===");
}
