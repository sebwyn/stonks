use bevy::{app::{App, Startup}, prelude::*, DefaultPlugins};
use data_viewer::DataViewerPlugin;

mod data_viewer;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DataViewerPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    
}


