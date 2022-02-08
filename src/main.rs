extern crate bevy_ecs_tilemap as tilemap;

use bevy::{
    core::Time,
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{
        mouse::{MouseButtonInput, MouseWheel},
        Input,
    },
    math::Vec3,
    prelude::*,
    render::render_resource::TextureUsages,
};
use tilemap::prelude::*;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Invincible"),
            ..Default::default()
        })
        .init_resource::<Game>()
        .add_state(GameState::Playing)
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(setup)
        .add_system(spawner)
        .add_system(set_texture_filters_to_nearest)
        .add_startup_system(set_texture_filters_to_nearest)
        .add_system(delete_block_system)
        .run();
}

#[derive(Component)]
struct WorldCamera;

#[derive(Default)]
struct Game {
    world_map: WorldMap,
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
}

#[derive(Debug, Clone)]
struct WorldMap {
    map: Option<Entity>,
    map_id: u16,
    ground_layer_id: u16,
    start_coord: (f32, f32),
    map_size: MapSize,
    chunk_size: ChunkSize,
    tile_size: TileSize,
    texture_size: TextureSize,
}

impl WorldMap {
    fn get_tile_pos(&self, world_coord: Vec2) -> TilePos {
        let [world_x, world_y] = world_coord.to_array();
        let tile_pos_x = (world_x - self.start_coord.0) / self.tile_size.0;
        let tile_pos_y = (world_y - self.start_coord.1) / self.tile_size.1;

        TilePos(tile_pos_x as u32, tile_pos_y as u32)
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        Self {
            map: None,
            map_id: 0,
            ground_layer_id: 0,
            start_coord: (-32.0, -32.0),
            map_size: MapSize(1, 1),
            chunk_size: ChunkSize(32, 32),
            tile_size: TileSize(16., 16.),
            texture_size: TextureSize(32., 16.),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Playing,
}

#[derive(Default, Clone)]
struct Cell {}

fn setup(
    mut commands: Commands,
    mut game: ResMut<Game>,
    asset_server: Res<AssetServer>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(WorldCamera);

    let texture_handle = asset_server.load("ground_map.png");

    let map_entity = commands.spawn().id();
    let mut map = Map::new(game.world_map.map_id, map_entity);

    let map_settings = LayerSettings::new(
        game.world_map.map_size,
        game.world_map.chunk_size,
        game.world_map.tile_size,
        game.world_map.texture_size,
    );
    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        map_settings,
        game.world_map.map_id,
        game.world_map.ground_layer_id,
    );

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let map_size = MapSize(
        game.world_map.map_size.0 * game.world_map.chunk_size.0,
        game.world_map.map_size.1 * game.world_map.chunk_size.1,
    );
    for x in 0..map_size.0 {
        for y in 0..map_size.1 {
			println!("Tile pos: ({}, {})", x, y);
            let _ = layer_builder.set_tile(
                TilePos(x, y),
                Tile {
                    texture_index: rng.gen_range(0..2),
                    ..Default::default()
                }
                .into(),
            );
        }
    }

    map.add_layer(&mut commands, game.world_map.ground_layer_id, layer_entity);
    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands.entity(layer_entity);

    game.world_map.map = Some(map_entity);
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(
            game.world_map.start_coord.0,
            game.world_map.start_coord.1,
            0.,
        ))
        .insert(GlobalTransform::default());
}

fn spawner(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection)>,
) {
    for (mut transform, mut ortho) in camera_query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            direction += Vec3::new(0., 1., 0.);
        };
        if keyboard.pressed(KeyCode::A) {
            direction += Vec3::new(-1., 0., 0.);
        };
        if keyboard.pressed(KeyCode::D) {
            direction += Vec3::new(1., 0., 0.);
        };
        if keyboard.pressed(KeyCode::S) {
            direction += Vec3::new(0., -1., 0.);
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

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
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
		if let Some(real_pos) = resolve_cursor_position(windows, world_camera) {
			let [x, y] = real_pos.to_array();
			println!("{} {}", x, y);
			let tile_pos = game.world_map.get_tile_pos(Vec2::new(x, y));
			println!("{:?}", tile_pos);

			let tile_entity = map_query.get_tile_entity(
				tile_pos,
				game.world_map.map_id,
				game.world_map.ground_layer_id,
			);
			println!("Params = {:?} {} {}", tile_pos,
			game.world_map.map_id,
			game.world_map.ground_layer_id);
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

fn resolve_cursor_position(wnds: Res<Windows>, q_camera: Query<&Transform, With<WorldCamera>>) -> Option<Vec2> {
    let wnd = wnds.get_primary().unwrap();

    if let Some(pos) = wnd.cursor_position() {
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        let p = pos - size / 2.0;

        let camera_transform = q_camera.single();
		println!("Camera transform = {:?}", camera_transform);

        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        
		Some(pos_wld.truncate().truncate())
    } else {
		None
	}
}
