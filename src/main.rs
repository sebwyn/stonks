use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;

use bevy::render::render_resource::encase::vector::FromVectorParts;
use bevy::window::PrimaryWindow;
use bevy::{app::App, DefaultPlugins};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UserCommands::default())
        .add_systems(Startup, create_graph)
        .add_systems(Update, pan)
        // .add_systems(Update, update_grid)
        .run();
}

#[derive(Resource, Default)]
struct UserCommands {
    pan_state: Option<(Vec2, Vec3)>
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct GridLine;

#[derive(Component)]
struct Grid {}

fn create_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    }, MainCamera));

    commands.spawn((ColorMesh2dBundle {
        mesh: meshes.add(Rectangle::default()).into(),
        material: materials.add(Color::from(PURPLE)),
        transform: Transform::default().with_scale(Vec3::new(10., 600., 1.)),
        ..Default::default()
    }, GridLine));
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

// fn update_grid(camera_projection_query: Query<&OrthographicProjection, (With<MainCamera>, Changed<OrthographicProjection>)>, grid_lines: Query<Entity, With<GridLine>>) {
//     if let Ok(camera_projection) = camera_projection_query.get_single() {
//         camera_projection.scale

//         camera_projection.area
//     }
// }