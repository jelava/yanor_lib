use bevy::{log::LogPlugin, prelude::*, state::app::StatesPlugin};

pub struct BaseTestPlugins;

impl Plugin for BaseTestPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalPlugins, StatesPlugin));
    }
}

pub fn end_test(mut app_exit: MessageWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
