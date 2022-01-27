use bevy::{input::{Input, mouse::MouseWheel}, math::Vec3, prelude::*, core::Time};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: String::from("Invincible"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_startup_system(startup)
        .add_system(spawner)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

#[derive(Component)]
struct WorldCamera;

#[derive(Component)]
struct CoordText;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("njfviujv");
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(TextBundle {
            style: Style {
				align_self: AlignSelf::FlexStart,
				position: Rect {
					bottom: Val::Percent(50.),
					..Default::default()
				},
                ..Default::default()
            },
            text: Text::with_section(
                "RUST - IMBAAAAAAA",
                TextStyle {
                    font: asset_server.load("fonts/Dongle-Regular.ttf"),
                    font_size: 50.,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            ..Default::default()
        })
        .insert(CoordText);

    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(WorldCamera);
    let image: Handle<Image> = asset_server.load("картинка.png");
    commands.spawn_bundle(SpriteBundle {
        texture: image,
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            scale: Vec3::new(0.5, 0.5, 0.),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn spawner(
	time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
	mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<WorldCamera>>,
    mut text_query: Query<&mut Text, With<CoordText>>,
	asset_server: Res<AssetServer>
) {
    for (mut transform, mut ortho) in camera_query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::W) {
            direction += Vec3::new(0., -2., 0.);
        };
		if keyboard.pressed(KeyCode::A) {
            direction += Vec3::new(2., 0., 0.);
        };
		if keyboard.pressed(KeyCode::D) {
            direction += Vec3::new(-2., 0., 0.);
        };
        if keyboard.pressed(KeyCode::S) {
            direction += Vec3::new(0., 2., 0.);
        }

        transform.translation += time.delta_seconds() * direction * 500.;
		text_query.get_single_mut().unwrap().sections = vec![TextSection {
			value: format!("{}", transform.translation.truncate()),
			style: TextStyle {
				font: asset_server.load("fonts/Dongle-Regular.ttf"),
				font_size: 50.,
				color: Color::WHITE,
			},
		}];

		for event in mouse_wheel_events.iter() {
			ortho.scale -= event.y * 0.1;
		}	
    }
}
