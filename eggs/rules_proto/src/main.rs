mod factions;
mod rules;

use bevy::{log::LogPlugin, prelude::*};

use crate::rules::{actions::*, conditions::*, *};

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LogPlugin::default()))
        .add_plugins(RulesPlugin)
        .add_systems(Startup, spawn_stuff)
        .run();

    fn spawn_stuff(mut commands: Commands) {
        let n = 1;

        for _i in 0..n {
            commands.spawn((
                Actor {
                    energy: 0,
                    max_energy: 150,
                    speed: 100,
                },
                Ruleset {
                    points: vec![vec![3, 3], vec![1, 2], vec![2, 1], vec![1, 1]],
                },
                Condition {
                    index: 0,
                    condition: Check::<2>,
                },
                Condition {
                    index: 1,
                    condition: Check::<3>,
                },
                Condition {
                    index: 2,
                    condition: Check::<5>,
                },
                Condition {
                    index: 3,
                    condition: Check::<7>,
                },
                Action {
                    index: 0,
                    action: Do::<2>,
                },
                Action {
                    index: 1,
                    action: Do::<3>,
                },
                Fallback { action: Do::<0> },
            ));
        }

        /*
        for _i in 0..n {
            commands.spawn((
                Actor {
                    energy: 0,
                    max_energy: 150,
                    speed: 100,
                },
                Ruleset {
                    points: vec![vec![3, 3], vec![1, 2], vec![2, 1], vec![1, 1]],
                },
                Condition {
                    index: 0,
                    condition: Check::<7>,
                },
                Condition {
                    index: 1,
                    condition: Check::<5>,
                },
                Condition {
                    index: 2,
                    condition: Check::<3>,
                },
                Condition {
                    index: 3,
                    condition: Check::<2>,
                },
                Action {
                    index: 0,
                    action: Do::<3>,
                },
                Action {
                    index: 1,
                    action: Do::<2>,
                },
                Fallback { action: Do::<1> },
            ));
        }
        */
    }
}
