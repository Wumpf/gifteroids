use std::time::Duration;

use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    gifteroids::{spawn_gifteroid, GiftSprites, GifteroidSize},
    spaceship::Snowball,
    DespawnOnStateEnter, GameState, MovementSpeed,
};

const SANTA_SPRITE_SIZE: f32 = 128.0;

pub struct SantaPlugin;

impl Plugin for SantaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SantaDestroyedEvent>()
            .add_startup_system(on_load)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(santa_reset_timer))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(santa_shoot)
                    .with_system(santa_snowball_collision)
                    .with_system(santa_spawn)
                    .with_system(santa_despawn),
            );
    }
}

#[derive(Resource)]
struct SantaSprite(Handle<Image>);

#[derive(Resource)]
struct SantaSpawner {
    last_spawn: Duration,
    gen_number: u64,
}

#[derive(Component)]
struct Santa {
    last_gifteroid_spawn: Duration,
    gen_number: u64,
}

pub struct SantaDestroyedEvent;

fn on_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SantaSprite(asset_server.load("santa.png")));
}

fn santa_reset_timer(mut commands: Commands, time: Res<Time>) {
    commands.insert_resource(SantaSpawner {
        last_spawn: time.elapsed(),
        gen_number: 0,
    })
}

fn santa_shoot(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut Santa)>,
    time: Res<Time>,
    gift_sprite: Res<GiftSprites>,
) {
    const SECONDS_BETWEEN_GIFTEROID_SPAWN: f32 = 0.8;

    for (transform, mut santa) in &mut query {
        if (time.elapsed() - santa.last_gifteroid_spawn).as_secs_f32()
            > SECONDS_BETWEEN_GIFTEROID_SPAWN
        {
            santa.last_gifteroid_spawn = time.elapsed();
            santa.gen_number += 1;

            let mut rng = StdRng::seed_from_u64(santa.gen_number);

            spawn_gifteroid(
                &mut commands,
                gift_sprite.gift0.clone(),
                transform.translation.truncate(),
                &mut rng,
                GifteroidSize::Medium,
            );
        }
    }
}

fn santa_snowball_collision(
    mut commands: Commands,
    query_snowballs: Query<(Entity, &Transform), With<Snowball>>,
    query_santa: Query<(Entity, &Transform), With<Santa>>,
    mut destroyed_events: EventWriter<SantaDestroyedEvent>,
) {
    for (entity_santa, transform_santa) in &query_santa {
        let aabb = Rect::from_center_half_size(
            transform_santa.translation.truncate(),
            Vec2::new(SANTA_SPRITE_SIZE * 0.5, SANTA_SPRITE_SIZE * 0.125),
        );

        for (entity_snowball, transform_snowball) in &query_snowballs {
            if aabb.contains(transform_snowball.translation.truncate()) {
                commands.entity(entity_santa).despawn();
                commands.entity(entity_snowball).despawn();

                destroyed_events.send(SantaDestroyedEvent);
            }
        }
    }
}

fn santa_spawn(
    mut commands: Commands,
    mut spawn_timer: ResMut<SantaSpawner>,
    sprite: Res<SantaSprite>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    const SECONDS_BETWEEN_SPAWNS: f32 = 8.0;
    if (time.elapsed() - spawn_timer.last_spawn).as_secs_f32() < SECONDS_BETWEEN_SPAWNS {
        return;
    }

    const SANTA_SPEED: f32 = 300.0;

    spawn_timer.last_spawn = time.elapsed();
    spawn_timer.gen_number += 1;

    let mut rng = StdRng::seed_from_u64(12345 + spawn_timer.gen_number);
    let left = rng.gen();

    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    let x_pos = (screen_size.x + SANTA_SPRITE_SIZE) * 0.5;
    let (x_pos, movement) = if left {
        (-x_pos, Vec2::new(SANTA_SPEED, 0.0))
    } else {
        (x_pos, Vec2::new(-SANTA_SPEED, 0.0))
    };

    commands
        .spawn(SpriteBundle {
            texture: sprite.0.clone(),
            transform: Transform {
                translation: Vec3::new(
                    x_pos,
                    rng.gen_range(
                        (-screen_size.y + SANTA_SPRITE_SIZE)..(screen_size.y - SANTA_SPRITE_SIZE),
                    ) * 0.5,
                    0.0,
                ),
                scale: Vec3::new(1.0 - 2.0 * (left as i32 as f32), 1.0, 1.0),
                rotation: Quat::IDENTITY,
            },
            ..default()
        })
        .insert(MovementSpeed(movement))
        .insert(Santa {
            last_gifteroid_spawn: time.elapsed(),
            gen_number: spawn_timer.gen_number * 10,
        })
        .insert(DespawnOnStateEnter(GameState::Game));
}

fn santa_despawn(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Santa>>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    for (entity, transform) in &query {
        if transform.translation.x > screen_size.x * 0.5 + SANTA_SPRITE_SIZE
            || transform.translation.x < -screen_size.x * 0.5 - SANTA_SPRITE_SIZE
        {
            commands.entity(entity).despawn();
        }
    }
}
