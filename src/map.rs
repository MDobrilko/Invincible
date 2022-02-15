use bevy::{
    prelude::*,
    render::render_resource::TextureUsages
};
use bevy_ecs_tilemap::prelude::*;

type LayerId = u16;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_startup_system(set_texture_filters_to_nearest);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    let mut world_map = WorldMap::default();

    let map_entity = commands.spawn().id();
    let mut map = Map::new(world_map.map_id, map_entity);

    let layers = vec![
        world_map.setup_ground_layer(&mut commands, &asset_server, &mut map_query),
        world_map.setup_building_layer(&mut commands, &asset_server, &mut map_query),
    ];
    map.add_layers(&mut commands, layers.into_iter());

    world_map.map = Some(map_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(
            world_map.start_coord.0,
            world_map.start_coord.1,
            0.,
        ))
        .insert(GlobalTransform::default());
}

#[derive(Component, Debug, Clone)]
pub struct WorldMap {
    map: Option<Entity>,
    pub map_id: u16,
    pub ground_layer_id: LayerId,
    pub building_layer_id: LayerId,
    start_coord: (f32, f32),
    map_size: MapSize,
    chunk_size: ChunkSize,
    tile_size: TileSize,
    texture_size: TextureSize,
}

impl WorldMap {
    fn setup_ground_layer(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, map_query: &mut MapQuery) -> (LayerId, Entity) {
        let texture_handle = asset_server.load("tiles_map.png");

        let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
            commands,
            self.get_settings(),
            self.map_id,
            self.ground_layer_id,
        );
    
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let map_size = MapSize(
            self.map_size.0 * self.chunk_size.0,
            self.map_size.1 * self.chunk_size.1,
        );
        for x in 0..map_size.0 {
            for y in 0..map_size.1 {
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

        map_query.build_layer(commands, layer_builder, texture_handle);
        commands.entity(layer_entity);

        (self.ground_layer_id, layer_entity)
    }

    fn setup_building_layer(&self, commands: &mut Commands, asset_server: &Res<AssetServer>, map_query: &mut MapQuery) -> (LayerId, Entity) {
        let texture_handle = asset_server.load("tiles_map.png");

        let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
            commands,
            self.get_settings(),
            self.map_id,
            self.building_layer_id,
        );

        let _ = layer_builder.set_tile(
            TilePos(16, 16),
            Tile {
                texture_index: 2,
                ..Default::default()
            }
            .into()
        );

        map_query.build_layer(commands, layer_builder, texture_handle);
        commands.entity(layer_entity);

        (self.building_layer_id, layer_entity)
    }

    fn get_settings(&self) -> LayerSettings {
        LayerSettings::new(
            self.map_size,
            self.chunk_size,
            self.tile_size,
            self.texture_size,
        )
    }

    pub fn get_tile_pos(&self, world_coord: Vec2) -> Option<TilePos> {
        let [world_x, world_y] = world_coord.to_array();
        let tile_pos_x = (world_x - self.start_coord.0) / self.tile_size.0;
        let tile_pos_y = (world_y - self.start_coord.1) / self.tile_size.1;

        if tile_pos_x < 0. || tile_pos_y < 0. {
            return None;
        }
        if tile_pos_x > (self.chunk_size.0 * self.map_size.0) as f32 {
            return None;
        }
        if tile_pos_y > (self.chunk_size.1 * self.map_size.1) as f32 {
            return None;
        }

        Some(TilePos(tile_pos_x as u32, tile_pos_y as u32))
    }
}

impl Default for WorldMap {
    fn default() -> Self {
        Self {
            map: None,
            map_id: 0,
            ground_layer_id: 0,
            building_layer_id: 1,
            start_coord: (-32.0, -32.0),
            map_size: MapSize(1, 1),
            chunk_size: ChunkSize(32, 32),
            tile_size: TileSize(16., 16.),
            texture_size: TextureSize(48., 16.),
        }
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
