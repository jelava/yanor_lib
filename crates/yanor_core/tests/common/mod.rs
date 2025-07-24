use bevy::{log::LogPlugin, prelude::*, state::app::StatesPlugin};

pub struct BaseTestPlugins;

impl Plugin for BaseTestPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((MinimalPlugins, LogPlugin::default(), StatesPlugin));
    }
}

pub fn end_test(mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
