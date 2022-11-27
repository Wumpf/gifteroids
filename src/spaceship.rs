use std::time::Duration;

use bevy::prelude::*;

use crate::{DespawnOnStateEnter, GameState, MovementSpeed};

pub struct SpaceshipPlugin;
pub struct SpaceShipDestroyedEvent {
    pub lives_left_before_destroy: u32,
}

// could ofc read this from data, but needlessly nasty to pass around
pub const NUM_LIVES_ON_STARTUP: u32 = 4;
pub const SPACESHIP_SPRITE_FILE: &str = "spaceship.png";
const SPACESHIP_SPRITE_SIZE: f32 = 128.0;
const SPAWN_INVINCIBLE_TIMER: f32 = 2.0;

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpaceShipDestroyedEvent>()
            .add_startup_system(on_load)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(initial_spawn))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(control_spaceship)
                    .with_system(snowballs_shoot)
                    .with_system(snowballs_screen_wrap)
                    .with_system(snowballs_timeout)
                    .with_system(on_space_ship_destroy)
                    .with_system(invincibility),
            );
    }
}

fn on_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    let space_ship_sprite = SpaceShipSprite(asset_server.load(SPACESHIP_SPRITE_FILE));
    commands.insert_resource(space_ship_sprite);
    commands.insert_resource(SnowballSprite(asset_server.load("snowball.png")));
}

fn initial_spawn(mut commands: Commands, space_ship_sprite: Res<SpaceShipSprite>) {
    spawn_spaceship(&mut commands, &space_ship_sprite, NUM_LIVES_ON_STARTUP);
}

fn spawn_spaceship(commands: &mut Commands, space_ship_sprite: &SpaceShipSprite, lives_left: u32) {
    commands
        .spawn_empty()
        .insert(SpaceShip {
            state: SpaceShipState::Invincible(Duration::from_secs_f32(SPAWN_INVINCIBLE_TIMER)),
            lives_left,
        })
        .insert(SnowballShootingCooldown(0.0))
        .insert(MovementSpeed(Vec2::ZERO))
        .insert(SpriteBundle {
            texture: space_ship_sprite.0.clone(),
            transform: Transform {
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(DespawnOnStateEnter(GameState::Any));
}

pub enum SpaceShipState {
    Normal,
    Invincible(Duration),
    Destroyed,
}

#[derive(Component)]
pub struct SpaceShip {
    pub state: SpaceShipState,
    pub lives_left: u32,
}

#[derive(Resource)]
pub struct SpaceShipSprite(pub Handle<Image>);

impl SpaceShip {
    pub fn bounding_triangle(transform: &Transform) -> (Vec2, Vec2, Vec2) {
        let position = transform.translation.truncate();
        let scale = transform.scale.x;

        let forward = (transform.rotation * Vec3::Y).truncate();
        let side = (transform.rotation * Vec3::X).truncate();

        let a = position + forward * (SPACESHIP_SPRITE_SIZE * 0.5 * scale);
        let b = position - (forward - side) * (SPACESHIP_SPRITE_SIZE * 0.5 * scale);
        let c = position - (forward + side) * (SPACESHIP_SPRITE_SIZE * 0.5 * scale);

        (a, b, c)
    }
}

#[derive(Component)]
struct SnowballShootingCooldown(f32);

#[derive(Component)]
pub struct Snowball {
    spawn_time: Duration,
}

#[derive(Resource)]
struct SnowballSprite(Handle<Image>);

fn control_spaceship(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut MovementSpeed, &mut Transform), With<SpaceShip>>,
) {
    const ACCELERATION: f32 = 400.0;
    const ROTATION_SPEED: f32 = 2.0;
    const FRICTION: f32 = 0.8;

    if query.get_single().is_err() {
        return;
    }

    let (mut speed, mut transform) = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        transform.rotate_z(ROTATION_SPEED * time.delta_seconds());
        transform.rotation = transform.rotation.normalize();
    }
    if keyboard_input.pressed(KeyCode::Right) {
        transform.rotate_z(-ROTATION_SPEED * time.delta_seconds());
        transform.rotation = transform.rotation.normalize();
    }
    if keyboard_input.pressed(KeyCode::Up) {
        speed.0 +=
            transform.rotation.mul_vec3(Vec3::Y).truncate() * (ACCELERATION * time.delta_seconds());
    }
    speed.0 *= FRICTION.powf(time.delta_seconds());

    let camera = camera_query.single();
    screen_wrap_space_ship(&mut transform, camera);
}

fn screen_wrap_space_ship(transform: &mut Transform, camera: &OrthographicProjection) {
    let (tri_a, tri_b, tri_c) = SpaceShip::bounding_triangle(transform);

    let max_x = tri_a.x.max(tri_b.x).max(tri_c.x);
    let min_x = tri_a.x.min(tri_b.x).min(tri_c.x);
    let max_y = tri_a.y.max(tri_b.y).max(tri_c.y);
    let min_y = tri_a.y.min(tri_b.y).min(tri_c.y);

    if max_y < camera.bottom {
        transform.translation.y = camera.top + (transform.translation.y - min_y) - 0.1;
    } else if min_y > camera.top {
        transform.translation.y = camera.bottom - (max_y - transform.translation.y) + 0.1;
    }
    if max_x < camera.left {
        transform.translation.x = camera.right + (transform.translation.x - min_x) - 0.1;
    } else if min_x > camera.right {
        transform.translation.x = camera.left - (max_x - transform.translation.x) + 0.1;
    }
}

fn snowballs_shoot(
    mut commands: Commands,
    time: Res<Time>,
    snowball_sprite: Res<SnowballSprite>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut SnowballShootingCooldown, &Transform)>,
) {
    const SNOWBALL_COOLDOWN_SECONDS: f32 = 0.25;
    const SNOWBALL_SPEED: f32 = 500.0;

    if query.get_single().is_err() {
        return;
    }

    let (mut cooldown, transform) = query.single_mut();
    cooldown.0 -= time.delta_seconds();
    cooldown.0 = cooldown.0.min(0.0);

    if cooldown.0 > 0.0 || !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    let (tri_a, _, _) = SpaceShip::bounding_triangle(transform);

    cooldown.0 += SNOWBALL_COOLDOWN_SECONDS;
    commands
        .spawn_empty()
        .insert(Snowball {
            spawn_time: time.elapsed(),
        })
        .insert(MovementSpeed(
            transform.rotation.mul_vec3(Vec3::Y).truncate() * SNOWBALL_SPEED,
        ))
        .insert(SpriteBundle {
            texture: snowball_sprite.0.clone(),
            transform: Transform {
                translation: tri_a.extend(0.0),
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(DespawnOnStateEnter(GameState::Any));
}

fn snowballs_screen_wrap(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut query: Query<&mut Transform, With<Snowball>>,
) {
    const SNOWBALL_SIZE: f32 = 8.0;

    let camera = camera_query.single();

    for mut transform in &mut query {
        if transform.translation.y + SNOWBALL_SIZE < camera.bottom {
            transform.translation.y = camera.top + SNOWBALL_SIZE - 0.1;
        } else if transform.translation.y - SNOWBALL_SIZE > camera.top {
            transform.translation.y = camera.bottom - SNOWBALL_SIZE + 0.1;
        }
        if transform.translation.x + SNOWBALL_SIZE < camera.left {
            transform.translation.x = camera.right + SNOWBALL_SIZE - 0.1;
        } else if transform.translation.x - SNOWBALL_SIZE > camera.right {
            transform.translation.x = camera.left - SNOWBALL_SIZE + 0.1;
        }
    }
}

fn snowballs_timeout(
    mut commands: Commands,
    time: Res<Time>,
    snowballs: Query<(Entity, &Snowball)>,
) {
    const SNOWBALL_MAX_LIFE_TIME: Duration = Duration::new(1, 500_000_000);

    let min_snowball_time = time.elapsed().saturating_sub(SNOWBALL_MAX_LIFE_TIME);

    for (entity, snowball) in &snowballs {
        if snowball.spawn_time < min_snowball_time {
            commands.entity(entity).despawn();
        }
    }
}

fn on_space_ship_destroy(
    mut commands: Commands,
    mut destroyed_events: EventReader<SpaceShipDestroyedEvent>,
    space_ship_sprite: Res<SpaceShipSprite>,
    query_spaceship: Query<(Entity, With<SpaceShip>)>,
) {
    let Some(destroyed_event) = destroyed_events.iter().next() else {
        return;
    };

    let entity = query_spaceship.single().0;
    commands.entity(entity).despawn();

    if destroyed_event.lives_left_before_destroy > 0 {
        spawn_spaceship(
            &mut commands,
            &space_ship_sprite,
            destroyed_event.lives_left_before_destroy - 1,
        );
    }
}

fn invincibility(mut query: Query<(&mut SpaceShip, &mut Sprite)>, time: Res<Time>) {
    if let Ok((mut ship, mut sprite)) = query.get_single_mut() {
        if let SpaceShipState::Invincible(invincibility_time_left) = ship.state {
            if invincibility_time_left.checked_sub(time.delta()).is_none() {
                ship.state = SpaceShipState::Normal;
                sprite.color = Color::WHITE;
            } else {
                ship.state = SpaceShipState::Invincible(invincibility_time_left - time.delta());

                let brightness = if (time.elapsed_seconds() * 4.0).fract() > 0.5 {
                    0.75
                } else {
                    0.5
                };
                sprite.color = Color::rgba(brightness, brightness, brightness, 1.0);
            }
        } else {
            sprite.color = Color::WHITE;
        }
    }
}
