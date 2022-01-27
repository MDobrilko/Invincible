extern crate bevy_ecs_tilemap as tilemap;

use bevy::{
    core::Time,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{mouse::MouseWheel, Input},
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
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(setup)
        .add_system(spawner)
        .add_system(set_texture_filters_to_nearest)
        .add_startup_system(set_texture_filters_to_nearest)
        .run();
}

#[derive(Default)]
struct Game {
    map: Map,
    camera_should_focus: Vec3,
    camera_is_focus: Vec3,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum GameState {
    Playing,
}

#[derive(Default, Clone)]
struct Cell {}

fn setup(
    mut commands: Commands,
    game: ResMut<Game>,
    asset_server: Res<AssetServer>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("ground_map.png");

    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let mut map_settings = LayerSettings::new(
        MapSize(4, 4),
        ChunkSize(32, 32),
        TileSize(16.0, 16.0),
        TextureSize(32.0, 16.0),
    );
    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings, 0u16, 0u16);

	layer_builder.set_all(TileBundle::default());

    use rand::Rng;
    let mut rng = rand::thread_rng();
    layer_builder.for_each_tiles_mut(|_, tile| {
        tile.as_mut().map(|bundle| {
            bundle.tile.texture_index = rng.gen_range(0..2);
        });
    });

	map.add_layer(&mut commands, 0, layer_entity);
    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands.entity(layer_entity);


    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-1024.0, -1024.0, 0.0))
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
            ortho.scale -= event.y * 0.1;
            if ortho.scale < 0. {
                ortho.scale = 0.;
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
