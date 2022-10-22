use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    collision::{line_line_test, point_in_box},
    spaceship::{Snowball, SpaceShip, SpaceShipDestroyedEvent},
    MovementSpeed,
};

pub struct GifteroidsPlugin;

impl Plugin for GifteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.exclusive_system())
            .add_startup_system(spawn_gifteroids)
            .add_system(gifteroid_snowball_collision)
            .add_system(gifteroid_spaceship_collision)
            .add_system(screen_wrap_obb_entities);
    }
}

struct GiftSprites {
    gift0: Handle<Image>,
}

#[derive(Component)]
pub struct OrientedBox {
    // Half axis from center. axis are in a right angle
    pub axis0: Vec2,
    pub axis1: Vec2,
}

#[derive(Bundle)]
struct GifteroidBundle {
    size: GifteroidSize,
    #[bundle]
    sprite: SpriteBundle,
    movement: MovementSpeed,
    obb: OrientedBox,
}

#[derive(Component, Clone, Copy)]
enum GifteroidSize {
    Large = 0,
    Medium = 1,
    Small = 2,
}

fn setup(world: &mut World) {
    // operate on world directly so that other startup systems have the GiftSprites resources available
    let asset_server = world.resource::<AssetServer>();
    world.insert_resource(GiftSprites {
        gift0: asset_server.load("gift.png"),
    });
}

fn spawn_gifteroids(windows: Res<Windows>, sprites: Res<GiftSprites>, mut commands: Commands) {
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    const GIFTEROIDS_SPAWN_COUNT: u32 = 8;
    const GIFTEROIDS_SPAWN_PLAYER_CLEARANCE: f32 = 80.0;

    let mut rng = StdRng::seed_from_u64(123); // Fixed seed so we get the same start conditions every time.
    let half_screen_size = screen_size * 0.5;
    for _ in 0..GIFTEROIDS_SPAWN_COUNT {
        // random position. Leave space around the player.
        let position = Vec2 {
            x: rng.gen_range(GIFTEROIDS_SPAWN_PLAYER_CLEARANCE..half_screen_size.x)
                * if rng.gen::<bool>() { -1.0 } else { 1.0 },
            y: rng.gen_range(GIFTEROIDS_SPAWN_PLAYER_CLEARANCE..half_screen_size.y)
                * if rng.gen::<bool>() { -1.0 } else { 1.0 },
        };

        let movement_angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let sprite_angle = rng.gen_range(0.0..std::f32::consts::TAU);

        spawn_gifteroid(
            &mut commands,
            sprites.gift0.clone(),
            position,
            movement_angle,
            sprite_angle,
            GifteroidSize::Large,
        )
    }
}

fn spawn_gifteroid(
    commands: &mut Commands,
    texture: Handle<Image>,
    position: Vec2,
    movement_angle: f32,
    sprite_angle: f32,
    size: GifteroidSize,
) {
    const GIFTEROIDS_BASE_SPEED: f32 = 50.0;

    // manual measurement from gift.png
    const GIFTSPRITE_HALF_EXTENT_X: f32 = 46.0;
    const GIFTSPRITE_HALF_EXTENT_Y: f32 = 64.0;

    let speed = GIFTEROIDS_BASE_SPEED * (size as i32 + 1) as f32; // TODO: Vary this?
    let scale = 0.5_f32.powi(size as i32);
    let movement = Vec2::new(movement_angle.cos(), movement_angle.sin()) * speed;

    let sprite_x_dir = Vec2::new(sprite_angle.cos(), sprite_angle.sin());
    let obb = OrientedBox {
        axis0: Vec2::new(GIFTSPRITE_HALF_EXTENT_X * scale, 0.0).rotate(sprite_x_dir),
        axis1: Vec2::new(0.0, GIFTSPRITE_HALF_EXTENT_Y * scale).rotate(sprite_x_dir),
    };

    commands.spawn_bundle(GifteroidBundle {
        size,
        sprite: SpriteBundle {
            texture,
            transform: Transform {
                translation: position.extend(0.0),
                rotation: Quat::from_rotation_z(sprite_angle),
                scale: Vec3::new(scale, scale, 1.0),
            },
            ..default()
        },
        movement: MovementSpeed(movement),
        obb,
    });
}

fn screen_wrap_obb_entities(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut query: Query<(&mut Transform, &OrientedBox)>,
) {
    let camera = camera_query.single();

    for (mut transform, obb) in &mut query {
        let axis0_abs = obb.axis0.abs();
        let axis1_abs = obb.axis1.abs();
        let max_axis_step = axis0_abs + axis1_abs;
        let min = transform.translation.truncate() - max_axis_step;
        let max = transform.translation.truncate() + max_axis_step;
        let size = (max - min).abs();

        if max.y < camera.bottom {
            transform.translation.y = camera.top + size.y * 0.5 - 0.1;
        } else if min.y > camera.top {
            transform.translation.y = camera.bottom - size.y * 0.5 + 0.1;
        }
        if max.x < camera.left {
            transform.translation.x = camera.right + size.x * 0.5 - 0.1;
        } else if min.x > camera.right {
            transform.translation.x = camera.left - size.x * 0.5 + 0.1;
        }
    }
}

fn gifteroid_snowball_collision(
    mut commands: Commands,
    sprites: Res<GiftSprites>,
    query_gifteroids: Query<(Entity, &Transform, &OrientedBox, &GifteroidSize)>,
    query_snowballs: Query<(Entity, &Transform), With<Snowball>>,
) {
    for (entity_gifteroid, transform_gifteroid, obb, size) in &query_gifteroids {
        let position_gifteroid = transform_gifteroid.translation.truncate();
        for (entity_snowball, transform_snowball) in &query_snowballs {
            // snowballs have a radius, but we ignore it here since they move fast enough
            if point_in_box(
                &obb,
                transform_snowball.translation.truncate(),
                position_gifteroid,
            ) {
                commands.entity(entity_gifteroid).despawn();
                commands.entity(entity_snowball).despawn();

                let new_size = match size {
                    GifteroidSize::Large => GifteroidSize::Medium,
                    GifteroidSize::Medium => GifteroidSize::Small,
                    GifteroidSize::Small => continue,
                };
                for _ in 0..2 {
                    spawn_gifteroid(
                        &mut commands,
                        sprites.gift0.clone(),
                        position_gifteroid,
                        rand::thread_rng().gen_range(0.0..std::f32::consts::TAU),
                        rand::thread_rng().gen_range(0.0..std::f32::consts::TAU),
                        new_size,
                    )
                }
            }
        }
    }
}

fn gifteroid_spaceship_collision(
    mut commands: Commands,
    query_gifteroids: Query<(&Transform, &OrientedBox), With<GifteroidSize>>,
    query_spaceship: Query<(Entity, &Transform), With<SpaceShip>>,
    mut destroyed_events: EventWriter<SpaceShipDestroyedEvent>,
) {
    // Detect collision by checking line collisions. Not perfect, but good enough and easy to implement
    // outer lines of spaceship
    if let Err(_) = query_spaceship.get_single() {
        return;
    }
    let (spaceship_entity, spaceship_transform) = query_spaceship.single();
    let (tri_a, tri_b, tri_c) = SpaceShip::bounding_triangle(spaceship_transform);

    for (transform_gifteroid, obb) in &query_gifteroids {
        let position_gifteroid = transform_gifteroid.translation.truncate();

        // outer lines of gifteroid
        let top_right = position_gifteroid + obb.axis0 + obb.axis1;
        let top_left = position_gifteroid - obb.axis0 + obb.axis1;
        let bottom_left = position_gifteroid - obb.axis0 - obb.axis1;
        let bottom_right = position_gifteroid + obb.axis0 - obb.axis1;

        if line_line_test(tri_a, tri_b, top_right, top_left)
            || line_line_test(tri_a, tri_b, top_left, bottom_left)
            || line_line_test(tri_a, tri_b, bottom_left, bottom_right)
            || line_line_test(tri_a, tri_b, bottom_right, top_right)
            || line_line_test(tri_b, tri_c, top_right, top_left)
            || line_line_test(tri_b, tri_c, top_left, bottom_left)
            || line_line_test(tri_b, tri_c, bottom_left, bottom_right)
            || line_line_test(tri_b, tri_c, bottom_right, top_right)
            || line_line_test(tri_c, tri_a, top_right, top_left)
            || line_line_test(tri_c, tri_a, top_left, bottom_left)
            || line_line_test(tri_c, tri_a, bottom_left, bottom_right)
            || line_line_test(tri_c, tri_a, bottom_right, top_right)
        {
            destroyed_events.send(SpaceShipDestroyedEvent(spaceship_entity));
            commands.entity(spaceship_entity).despawn();
            break;
        }
    }
}