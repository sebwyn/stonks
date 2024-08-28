use bevy::app::Plugin;
use bevy::color::palettes::css::{GREY, WHITE};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct DataViewerPlugin;

impl Plugin for DataViewerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(UserCommands::default());
        app.add_systems(Startup, setup);
        app.add_systems(Update, pan);
        app.add_systems(Update, prepare_grid);
        app.add_systems(Update, draw_grid);
    }
}

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    
) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    }, MainCamera));
    
    commands.spawn(Grid {
        minor_line_color: GREY,
        x_axis_line_color: WHITE,
        y_axis_line_color: WHITE,
        minor_line_width_px: 2,
        minor_line_frequency: 100.0
    });
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

    minor_line_color: Srgba,
    x_axis_line_color: Srgba,
    y_axis_line_color: Srgba
}

#[derive(Component)]
struct GridRenderable {
    line_mesh_prototype: ColorMesh2dBundle,

    minor_line_material: Handle<ColorMaterial>,
    x_axis_line_material: Handle<ColorMaterial>,
    y_axis_line_material: Handle<ColorMaterial>
}

fn prepare_grid(
    mut commands: Commands,
    grid_query: Query<(Entity, &Grid), (Added<Grid>, Without<GridRenderable>)>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Ok((entity, grid)) = grid_query.get_single() {
        commands.entity(entity).insert(GridRenderable {
            line_mesh_prototype: ColorMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                ..Default::default()
            },
            minor_line_material: materials.add(ColorMaterial::from(Color::from(grid.minor_line_color))),
            x_axis_line_material: materials.add(ColorMaterial::from(Color::from(grid.x_axis_line_color))),
            y_axis_line_material: materials.add(ColorMaterial::from(Color::from(grid.y_axis_line_color))),

        });
    }
}

#[derive(Component)]
struct GridLine;

fn draw_grid(
    mut commands: Commands,
    grid_query: Query<(&Grid, &GridRenderable)>,
    grid_lines: Query<Entity, With<GridLine>>,
    projection_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>
) {
    for grid_line_entity in grid_lines.into_iter() {
        commands.entity(grid_line_entity).despawn();
    }

    let Ok((grid_spec, grid_renderable)) = grid_query.get_single() else {
        return
    };

    let (camera_transform, camera_projection) = projection_query.single();
    

    let mut line = grid_renderable.line_mesh_prototype.clone();
    line.material = grid_renderable.minor_line_material.clone();

    let px_to_data = |px: i32| px as f32 / camera_projection.scale;

    let starting_vertical_line_index = ((camera_transform.translation.x - camera_projection.area.width()/2.0) / grid_spec.minor_line_frequency).ceil() as i32;
    let ending_vertical_line_index = ((camera_transform.translation.x + camera_projection.area.width()/2.0) / grid_spec.minor_line_frequency).floor() as i32;


    for i in starting_vertical_line_index..=ending_vertical_line_index {
        let center_rect_x = i as f32 * grid_spec.minor_line_frequency;
        let width = px_to_data(grid_spec.minor_line_width_px);
        
        let mut line = line.clone();
        if i == 0 { line.material = grid_renderable.y_axis_line_material.clone() }

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
        if i == 0 { line.material = grid_renderable.x_axis_line_material.clone() }

        line.transform = Transform::default()
            .with_translation(Vec3::new(camera_transform.translation.x, center_rect_y, 0.0))
            .with_scale(Vec3::new(camera_projection.area.width(), height, 1.0));

        commands.spawn((line, GridLine));
    }

}