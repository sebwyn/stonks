use std::collections::HashMap;

use bevy::app::Plugin;
use bevy::color::palettes::css::{BLUE, GREEN, GREY, PURPLE, RED, WHITE, YELLOW};
use bevy::prelude::*;

use bevy::sprite::Mesh2dHandle;
use bevy::window::PrimaryWindow;
use chrono::NaiveDate;

pub struct DataViewerPlugin;

impl Plugin for DataViewerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(UserCommands::default());
        app.insert_resource(TimeseriesDataRepository::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, plot_timeseries);
        app.add_systems(Update, pan);
        app.add_systems(Update, draw_grid);
    }
}

#[derive(Resource, Default)]
pub struct TimeseriesDataRepository {
    pub keyed_datasets: HashMap<String, Vec<(NaiveDate, f64)>>
}

#[derive(Component)]
pub struct Timeseries {
    pub data_key: String,
    pub(crate) line_color: PlotColor
}

fn plot_timeseries(
    mut commands: Commands,
    camera_projection: Query<&OrthographicProjection, With<MainCamera>>,
    timeseries_data_repository: Res<TimeseriesDataRepository>,
    timeseries_query: Query<&Timeseries, Added<Timeseries>>,

    palette: Res<Palette>,
    plot_rect: Res<PlotRect>
) {
    let map_time_to_x_axis = |time: NaiveDate| {
        (time - NaiveDate::from_yo_opt(2000, 1).unwrap()).num_days() as f32 / (365.0 / 2.0)
    };
    let px_to_data = |px: i32| px as f32 / camera_projection.single().scale;
    let data_rect_size = px_to_data(5);

    for timeseries in timeseries_query.into_iter() {
        let data = timeseries_data_repository.keyed_datasets.get(&timeseries.data_key)
            .unwrap_or_else(|| panic!("Could not find dataset {} for timeseries", timeseries.data_key));

        let data_point_mesh = ColorMesh2dBundle {
            mesh: plot_rect.0.clone(),
            material: palette.material_from_color(&timeseries.line_color),
            ..Default::default()
        };
        
        let data_meshes = data.iter().map(|data_point| {
            let mut data_point_mesh = data_point_mesh.clone();

            data_point_mesh.transform = Transform::default()
                .with_translation(Vec3::new(map_time_to_x_axis(data_point.0), data_point.1 as f32 * 8.0, 1.0))
                .with_scale(Vec3::splat(data_rect_size));
            
            data_point_mesh
        })
        .collect::<Vec<_>>();
            
        commands.spawn_batch(data_meshes)
    }
}

#[derive(Resource)]
pub struct PlotRect(Mesh2dHandle);

#[derive(PartialEq, Eq, Hash)]
pub enum PlotColor {
    Green,
    Grey,
    White,
    Purple,
    Yellow,
    Red,
    Blue,
}

#[derive(Resource)]
struct Palette {
    colors: HashMap<PlotColor, Handle<ColorMaterial>>
}

impl Palette {
    fn new(mut material: ResMut<Assets<ColorMaterial>>) -> Self {
        let mut colors = HashMap::new();
        colors.insert(PlotColor::Green, material.add(Color::from(GREEN)));
        colors.insert(PlotColor::Grey, material.add(Color::from(GREY)));
        colors.insert(PlotColor::White, material.add(Color::from(WHITE)));
        colors.insert(PlotColor::Purple, material.add(Color::from(PURPLE)));
        colors.insert(PlotColor::Yellow, material.add(Color::from(YELLOW)));
        colors.insert(PlotColor::Red, material.add(Color::from(RED)));
        colors.insert(PlotColor::Blue, material.add(Color::from(BLUE)));

        
        Self {
            colors
        }
    }

    fn material_from_color(&self, color: &PlotColor) -> Handle<ColorMaterial> {
        self.colors.get(color).cloned().expect("Palette does not store color")
    }
}


#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    }, MainCamera));
    

    commands.spawn(Grid {
        minor_line_color: PlotColor::Grey,
        x_axis_line_color: PlotColor::White,
        y_axis_line_color: PlotColor::Grey,
        minor_line_width_px: 2,
        minor_line_frequency: 100.0
    });

    let rectangle_mesh: Mesh2dHandle = meshes.add(Rectangle::default()).into();
    commands.insert_resource(PlotRect(rectangle_mesh));
    commands.insert_resource(Palette::new(materials))
}


#[derive(Resource, Default)]
struct UserCommands {
    pan_state: Option<(Vec2, Vec3)>
}

fn pan(
    buttons: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>, 
    mut user_commands: ResMut<UserCommands>,
    mut camera_transform_query: Query<&mut Transform, With<MainCamera>>

) {
    let mut camera_transform = camera_transform_query.single_mut();
    let Some(mouse_position) = window_query.single().cursor_position() else { return };

    if buttons.just_pressed(MouseButton::Left) {
        user_commands.pan_state = Some((mouse_position, camera_transform.translation));
    }
    
    let Some((initital_mouse_position, initial_camera_translation)) = user_commands.pan_state else { return };
    if buttons.just_released(MouseButton::Left) || buttons.pressed(MouseButton::Left) {
        let pan_translation = initital_mouse_position - mouse_position;
        camera_transform.translation = initial_camera_translation + Vec3::new(pan_translation.x, -pan_translation.y, 0.);
    }

    if buttons.just_released(MouseButton::Left) {
        user_commands.pan_state = None;
    }
}

#[derive(Component)]
struct Grid {
    minor_line_frequency: f32,
    minor_line_width_px: i32,

    minor_line_color: PlotColor,
    x_axis_line_color: PlotColor,
    y_axis_line_color: PlotColor
}

#[derive(Component)]
struct GridLine;

fn draw_grid(
    mut commands: Commands,
    grid_spec: Query<&Grid>,
    grid_lines: Query<Entity, With<GridLine>>,
    projection_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>,

    palette: Res<Palette>,
    plot_rect: Res<PlotRect>
) {
    for grid_line_entity in grid_lines.into_iter() {
        commands.entity(grid_line_entity).despawn();
    }

    let grid_spec = grid_spec.single();
    let (camera_transform, camera_projection) = projection_query.single();

    

    let line = ColorMesh2dBundle {
        mesh: plot_rect.0.clone(),
        material: palette.material_from_color(&grid_spec.minor_line_color),
        ..Default::default()
    };

    let px_to_data = |px: i32| px as f32 / camera_projection.scale;

    let starting_vertical_line_index = ((camera_transform.translation.x - camera_projection.area.width()/2.0) / grid_spec.minor_line_frequency).ceil() as i32;
    let ending_vertical_line_index = ((camera_transform.translation.x + camera_projection.area.width()/2.0) / grid_spec.minor_line_frequency).floor() as i32;


    for i in starting_vertical_line_index..=ending_vertical_line_index {
        let center_rect_x = i as f32 * grid_spec.minor_line_frequency;
        let width = px_to_data(grid_spec.minor_line_width_px);
        
        let mut line = line.clone();
        if i == 0 { line.material = palette.material_from_color(&grid_spec.y_axis_line_color) }

        line.transform = Transform::default()
            .with_translation(Vec3::new(center_rect_x, camera_transform.translation.y, 0.0))
            .with_scale(Vec3::new(width, camera_projection.area.height(), 1.0));

        commands.spawn((line, GridLine));
    }

    let starting_horizontal_line_index = ((camera_transform.translation.y - camera_projection.area.height()/2.0) / grid_spec.minor_line_frequency).ceil() as i32;
    let ending_horizontal_line_index = ((camera_transform.translation.y + camera_projection.area.height()/2.0) / grid_spec.minor_line_frequency).floor() as i32;

    for i in starting_horizontal_line_index..=ending_horizontal_line_index {
        let center_rect_y = i as f32 * grid_spec.minor_line_frequency;
        let height = px_to_data(grid_spec.minor_line_width_px);
        
        let mut line = line.clone();
        if i == 0 { line.material = palette.material_from_color(&grid_spec.x_axis_line_color) }

        line.transform = Transform::default()
            .with_translation(Vec3::new(camera_transform.translation.x, center_rect_y, 0.0))
            .with_scale(Vec3::new(camera_projection.area.width(), height, 1.0));

        commands.spawn((line, GridLine));
    }

}