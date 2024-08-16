use std::path::Path;

use bevy::{app::{App, Startup}, prelude::*, DefaultPlugins};
use chrono::NaiveDate;
use data_viewer::{DataViewerPlugin, PlotColor, Timeseries, TimeseriesDataRepository};
use polars::prelude::*;

mod data_viewer;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, DataViewerPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut timeseries_data_repository: ResMut<TimeseriesDataRepository>
) {
    let lf = LazyCsvReader::new(Path::new("assets/datasets/co-emissions-per-capita.csv"))
        .finish()
        .unwrap();

    let us_co2_emissions_df = lf
        .filter(col("Entity").eq(lit("United States")))
        .collect()
        .unwrap();

    let us_emissions = us_co2_emissions_df["Year"]
        .i64()
        .unwrap()
        .into_iter().zip(
            us_co2_emissions_df["Annual_CO2_emissions_per_capita"]
                .f64()
                .unwrap()
        )
        .filter_map(|data_point| {
            if let (Some(a), Some(b)) = data_point { 
                if let Some(date) = NaiveDate::from_yo_opt(a as i32, 1) {
                    return Some(( date, b)) 
                }
            }
            None
        })
        .collect::<Vec<_>>();

    timeseries_data_repository.keyed_datasets.insert(String::from("united_states_emissions"), us_emissions);

    commands.spawn(Timeseries {
        data_key: "united_states_emissions".to_string(),
        line_color: PlotColor::Green,
    });
}


