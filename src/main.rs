extern crate bevy_ecs_tilemap as tilemap;

use bevy::{
    core::Time,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{
        mouse::MouseWheel,
        Input,
    },
    math::Vec3,
    prelude::*,
};
use tilemap::prelude::*;

pub mod camera;
pub mod input;
pub mod map;

use camera::WorldCamera;
use input::resolve_cursor_position;
use map::{
    MapPlugin,
    WorldMap,
    set_texture_filters_to_nearest,
};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Invincible"),
            vsync: true,
            ..Default::default()
        })
        .init_resource::<Game>()
        .add_state(GameState::Playing)
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(MapPlugin)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(setup)
        .add_system(spawner)
        .add_system(set_texture_filters_to_nearest)
        .add_system(delete_block_system)
        .run();
}

pub struct Game {
    world_map: WorldMap,
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
}

impl FromWorld for Game {
    fn from_world(world: &mut World) -> Self {
        let map_query = world.query::<(&WorldMap)>();

        Self {
            world_map: WorldMap::default(),
            camera_is_focus: Vec3::new(0., 0., 0.),
            camera_should_focus: Vec3::new(0., 0., 0.),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Playing,
}

#[derive(Default, Clone)]
struct Cell {}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(WorldCamera);
}

fn spawner(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut Transform, With<WorldCamera>>,
) {
    let speed = 0.5;

    for mut transform in camera_query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            direction += Vec3::new(0., speed, 0.);
        }
        if keyboard.pressed(KeyCode::A) {
            direction += Vec3::new(-speed, 0., 0.);
        }
        if keyboard.pressed(KeyCode::D) {
            direction += Vec3::new(speed, 0., 0.);
        }
        if keyboard.pressed(KeyCode::S) {
            direction += Vec3::new(0., -speed, 0.);
        }

        for event in mouse_wheel_events.iter() {
            transform.scale -= Vec3::new(event.y * 0.1, event.y * 0.1, 0.);
            if transform.scale[0] < 0.1 {
                transform.scale = Vec3::new(0.1, 0.1, transform.scale[2]);
            }
        }
    
        transform.translation += time.delta_seconds() * direction * 250.;
    }
}

fn delete_block_system(
    mut commands: Commands,
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
	world_camera: Query<&Transform, With<WorldCamera>>,
    game: Res<Game>,
    mut map_query: MapQuery,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
		if let Some(real_pos) = resolve_cursor_position(windows, &world_camera) {
			let tile_pos = match game.world_map.get_tile_pos(real_pos) {
                Some(pos) => pos,
                None => return,
            };

			let tile_entity = map_query.get_tile_entity(
				tile_pos,
				game.world_map.map_id,
				game.world_map.ground_layer_id,
			);
            
			match tile_entity {
				Ok(entity) => {
					commands.entity(entity).insert(Tile {
						visible: false,
						..Default::default()
					});

					map_query.notify_chunk_for_tile(
						tile_pos,
						game.world_map.map_id,
						game.world_map.ground_layer_id,
					);
				}
				Err(error) => println!("{:?}", error),
			}
		}
    }
}