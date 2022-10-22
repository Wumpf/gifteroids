use bevy::{prelude::*, sprite::Anchor};

use crate::spaceship::{SPACESHIP_SPRITE_FILE, SPACESHIP_SPRITE_SIZE};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_life_display);
    }
}

fn setup_life_display(
    windows: Res<Windows>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let num_lifes = 3;

    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };
    let top_left_corner = Vec2::new(-screen_size.x * 0.5, screen_size.y * 0.5);

    let texture = asset_server.load(SPACESHIP_SPRITE_FILE);

    let scale = 0.35;

    for i in 0..num_lifes {
        commands.spawn().insert_bundle(SpriteBundle {
            texture: texture.clone(),
            transform: Transform {
                translation: top_left_corner.extend(1.0)
                    + Vec3::new(
                        SPACESHIP_SPRITE_SIZE * scale * i as f32 + 10.0 * (i + 1) as f32,
                        -10.0,
                        0.0,
                    ),
                scale: Vec3::new(scale, scale, 1.0),
                ..default()
            },
            sprite: Sprite {
                anchor: Anchor::TopLeft,
                ..default()
            },
            ..default()
        });
    }
    commands.spawn().insert_bundle(SpriteBundle {
        transform: Transform {
            translation: top_left_corner.extend(0.5),
            scale: Vec3::new(
                SPACESHIP_SPRITE_SIZE * (num_lifes as f32) * scale + (1 + num_lifes) as f32 * 10.0,
                SPACESHIP_SPRITE_SIZE * scale + 20.0,
                1.5,
            ),
            ..default()
        },
        sprite: Sprite {
            anchor: Anchor::TopLeft,
            color: Color::BLACK,
            ..default()
        },
        ..default()
    });
}
