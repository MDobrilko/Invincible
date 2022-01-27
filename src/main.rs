use bevy::{
    core::Time,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::{mouse::MouseWheel, Input},
    math::Vec3,
    prelude::*,
};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Invincible"),
            ..Default::default()
        })
        .init_resource::<Game>()
        .add_state(GameState::Playing)
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(setup)
        .add_system(spawner)
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

struct Map {
    size: (usize, usize),
    cells: Vec<Vec<Cell>>,
}

impl Default for Map {
    fn default() -> Self {
        let default_size = (128, 128);
        let cells = vec![vec![Cell::default(); default_size.1]; default_size.0];

        Self {
            size: default_size,
            cells,
        }
    }
}

#[derive(Default, Clone)]
struct Cell {}

fn setup(mut commands: Commands, game: ResMut<Game>, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let ground_textures: [Handle<Image>; 2] = [
        asset_server.load("ground12.png"),
        asset_server.load("ground22.png"),
    ];

    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let mut rng = thread_rng();
    for (i, rows) in game.map.cells.iter().enumerate() {
        for (j, cell) in rows.iter().enumerate() {
            commands.spawn_bundle(SpriteBundle {
                texture: ground_textures.choose(&mut rng).unwrap().clone(),
                transform: Transform {
                    translation: Vec3::new(i as f32 * 16., j as f32 * 16., 0.),
                    ..Default::default()
                },
                ..Default::default()
            });
        }
    }

    println!("{} {}", game.map.cells.len(), game.map.cells[0].len());
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
