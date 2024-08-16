use bevy::{app::App, DefaultPlugins};
use data_viewer::DataViewerPlugin;

mod data_viewer;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DataViewerPlugin))
        .run();
}


