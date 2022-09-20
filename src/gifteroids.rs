use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::MovementSpeed;

pub struct GifteroidsPlugin;

impl Plugin for GifteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_gifteroids)
            .add_system(screen_wrap_obb_entities);
    }
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

#[derive(Component)]
enum GifteroidSize {
    Large,
    Medium,
    Small,
}

fn spawn_gifteroids(
    windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    const GIFTEROIDS_SPAWN_COUNT: u32 = 8;
    const GIFTEROIDS_SPAWN_PLAYER_CLEARANCE: f32 = 80.0;
    const GIFTEROIDS_BASE_SPEED: f32 = 50.0;

    // manual measurement from gift.png
    const GIFTSPRITE_HALF_EXTENT_X: f32 = 46.0;
    const GIFTSPRITE_HALF_EXTENT_Y: f32 = 64.0;

    let mut rng = StdRng::seed_from_u64(123); // Fixed seed so we get the same start conditions every time.
    let half_screen_size = screen_size * 0.5;
    let gifteroid_texture = asset_server.load("gift.png");
    for _ in 0..GIFTEROIDS_SPAWN_COUNT {
        // random position. Leave space around the player.
        let translation = Vec3 {
            x: rng.gen_range(GIFTEROIDS_SPAWN_PLAYER_CLEARANCE..half_screen_size.x)
                * if rng.gen::<bool>() { -1.0 } else { 1.0 },
            y: rng.gen_range(GIFTEROIDS_SPAWN_PLAYER_CLEARANCE..half_screen_size.y)
                * if rng.gen::<bool>() { -1.0 } else { 1.0 },
            z: 0.0,
        };

        let sprite_orientation = rng.gen_range(0.0..std::f32::consts::TAU);
        let sprite_x_dir = Vec2::new(sprite_orientation.cos(), sprite_orientation.sin());
        let movement_orientation = rng.gen_range(0.0..std::f32::consts::TAU);
        let movement = Vec2::new(movement_orientation.cos(), movement_orientation.sin())
            * GIFTEROIDS_BASE_SPEED;

        let obb = OrientedBox {
            axis0: Vec2::new(GIFTSPRITE_HALF_EXTENT_X, 0.0).rotate(sprite_x_dir),
            axis1: Vec2::new(0.0, GIFTSPRITE_HALF_EXTENT_Y).rotate(sprite_x_dir),
        };

        commands.spawn_bundle(GifteroidBundle {
            size: GifteroidSize::Large,
            sprite: SpriteBundle {
                texture: gifteroid_texture.clone(),
                transform: Transform {
                    translation,
                    rotation: Quat::from_rotation_z(sprite_orientation),
                    ..default()
                },
                ..default()
            },
            movement: MovementSpeed(movement),
            obb,
        });
    }
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
