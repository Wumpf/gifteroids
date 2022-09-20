use std::time::Duration;

use bevy::prelude::*;

use crate::MovementSpeed;

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(control_spaceship)
            .add_system(snowballs_shoot)
            .add_system(snowballs_screen_wrap)
            .add_system(snowballs_timeout);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(SpaceShip)
        .insert(SnowballShootingCooldown(0.0))
        .insert(MovementSpeed(Vec2::ZERO))
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            transform: Transform {
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..default()
            },
            ..default()
        });

    commands.insert_resource(SnowballSprite(asset_server.load("snowball.png")));
}

#[derive(Component)]
pub struct SpaceShip;

impl SpaceShip {
    pub fn bounding_triangle(transform: &Transform) -> (Vec2, Vec2, Vec2) {
        // could ofc read this from data, but needlessly nasty to pass around
        const SPACESHIP_SPRITE_SIZE: f32 = 128.0;

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
struct Snowball {
    spawn_time: Duration,
}

struct SnowballSprite(Handle<Image>);

fn control_spaceship(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut MovementSpeed, &mut Transform), With<SpaceShip>>,
) {
    const ACCELERATION: f32 = 400.0;
    const ROTATION_SPEED: f32 = 2.0;
    const FRICTION: f32 = 0.5;

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

    let (mut cooldown, transform) = query.single_mut();
    cooldown.0 -= time.delta_seconds();
    cooldown.0 = cooldown.0.min(0.0);

    if cooldown.0 > 0.0 || !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    let (tri_a, _, _) = SpaceShip::bounding_triangle(transform);

    cooldown.0 += SNOWBALL_COOLDOWN_SECONDS;
    commands
        .spawn()
        .insert(Snowball {
            spawn_time: time.time_since_startup(),
        })
        .insert(MovementSpeed(
            transform.rotation.mul_vec3(Vec3::Y).truncate() * SNOWBALL_SPEED,
        ))
        .insert_bundle(SpriteBundle {
            texture: snowball_sprite.0.clone(),
            transform: Transform {
                translation: tri_a.extend(0.0),
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..default()
            },
            ..default()
        });
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

    let min_snowball_time = time
        .time_since_startup()
        .saturating_sub(SNOWBALL_MAX_LIFE_TIME);

    for (entity, snowball) in &snowballs {
        if snowball.spawn_time < min_snowball_time {
            commands.entity(entity).despawn();
        }
    }
}
