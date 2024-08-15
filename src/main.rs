use std::f32::consts::PI;

use bevy::color::palettes::css::{PURPLE, WHITE};
use bevy::prelude::*;

use bevy::render::camera;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::window::PrimaryWindow;
use bevy::{app::App, DefaultPlugins};
use bevy_infinite_grid::{InfiniteGrid, InfiniteGridBundle, InfiniteGridPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InfiniteGridPlugin))
        .insert_resource(UserCommands::default())
        .add_systems(Startup, create_graph)
        .add_systems(Update, pan)
        .add_systems(PostUpdate, update_grid)
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
struct Grid {
    minor_line_frequency: f32,
    minor_line_width_px: i32,
}


impl Default for Grid {
    fn default() -> Self {
        Self { minor_line_frequency: 50.0, minor_line_width_px: 5  }
    }
}

#[derive(Component)]
struct GridLine;

fn create_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 1.0)),
            ..Default::default()
        },
        ..Default::default()
    }, MainCamera));
    
    let rectangle_mesh: Mesh2dHandle = meshes.add(Rectangle::default()).into();

    commands.spawn((ColorMesh2dBundle {
        mesh: rectangle_mesh.clone(),
        material: materials.add(Color::from(PURPLE)),
        transform: Transform::default().with_scale(Vec3::new(10., 600., 1.)),
        ..Default::default()
    }, GridLine));

    commands.spawn(Grid::default());
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

fn update_grid(
    mut commands: Commands,
    grid_spec: Query<&Grid>,
    grid_lines: Query<Entity, With<GridLine>>,
    projection_query: Query<(&Transform, &OrthographicProjection), With<MainCamera>>
) {
    for grid_line_entity in grid_lines.into_iter() {
        commands.entity(grid_line_entity).despawn();
    }


    let (camera_transform, camera_projection) = projection_query.single();

    let center_x =  camera_transform.translation.x;
    let center_y = camera_transform.translation.y;
    let half_viewed_width = camera_projection.area.width() / 2.0;
    let half_viewed_height = camera_projection.area.height() / 2.0;


    let left = center_x - half_viewed_width;
    let right = center_x + half_viewed_width;

    let bottom = center_y - half_viewed_height;
    let top = center_y + half_viewed_height;

    let grid_spec = grid_spec.single();

    let starting_horizontal_line_index = (left / grid_spec.minor_line_frequency).ceil() as i32;
    let ending_horizontal_line_index = (right / grid_spec.minor_line_frequency).floor() as i32;

    let px_to_data = |px: i32| px as f32 / camera_projection.scale;

    for i in starting_horizontal_line_index..=ending_horizontal_line_index {
        let x1 = i as f32 * grid_spec.minor_line_frequency - px_to_data(grid_spec.minor_line_width_px);
        let y1 = bottom;
        
        let x2 = i as f32 * grid_spec.minor_line_frequency + px_to_data(grid_spec.minor_line_width_px);
        let y2 = top;
        
        let width = x2 - x1;
        let height = y2 - y1;

        // commands.spawn(ColorMesh2dBundle { 
        //     mesh: Rectangle,
        //     material: ColorMaterial::from_color(WHITE),
        //     transform: todo!(),
        //     global_transform: todo!(),
        //     visibility: todo!(),
        //     inherited_visibility: todo!(),
        //     view_visibility: todo!() 
        // });
    }

}