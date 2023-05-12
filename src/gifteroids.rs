use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    collision::{line_line_test, point_in_obb},
    spaceship::{Snowball, SpaceShip, SpaceShipDestroyedEvent, SpaceShipState},
    DespawnOnStateEnter, GameState, MovementSpeed,
};

pub struct GifteroidsPlugin;

impl Plugin for GifteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GifteroidDestroyedEvent>()
            .add_startup_system(on_load)
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(spawn_gifteroids))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(gifteroid_snowball_collision)
                    .with_system(gifteroid_spaceship_collision)
                    .with_system(screen_wrap_obb_entities)
                    .with_system(check_win_condition),
            );

        #[cfg(feature = "rerun")]
        {
            app.add_system_set(
                SystemSet::on_update(GameState::Game).with_system(send_collision_geom_to_rerun),
            );
        }
    }
}

#[derive(Resource)]
pub struct GiftSprites {
    pub gift0: Handle<Image>,
}

pub struct GifteroidDestroyedEvent {
    pub split_in_two: bool,
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
    despawner: DespawnOnStateEnter,
}

#[derive(Component, Clone, Copy)]
pub enum GifteroidSize {
    Large = 0,
    Medium = 1,
    Small = 2,
}

fn on_load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GiftSprites {
        gift0: asset_server.load("gift.png"),
    });
}

fn spawn_gifteroids(windows: Res<Windows>, sprites: Res<GiftSprites>, mut commands: Commands) {
    let window = windows.get_primary().unwrap();
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    const GIFTEROIDS_SPAWN_COUNT: u32 = 7;
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

        spawn_gifteroid(
            &mut commands,
            sprites.gift0.clone(),
            position,
            &mut rng,
            GifteroidSize::Large,
        )
    }
}

pub fn spawn_gifteroid(
    commands: &mut Commands,
    texture: Handle<Image>,
    position: Vec2,
    rand: &mut StdRng,
    size: GifteroidSize,
) {
    const GIFTEROIDS_BASE_SPEED: f32 = 60.0;

    // manual measurement from gift.png
    const GIFTSPRITE_HALF_EXTENT_X: f32 = 46.0;
    const GIFTSPRITE_HALF_EXTENT_Y: f32 = 64.0;

    let movement_angle = rand.gen_range(0.0..std::f32::consts::TAU);
    let sprite_angle = rand.gen_range(0.0..std::f32::consts::TAU);

    let speed = GIFTEROIDS_BASE_SPEED * (size as i32 + 1) as f32; // TODO: Vary this?
    let scale = 0.5_f32.powi(size as i32);
    let movement = Vec2::new(movement_angle.cos(), movement_angle.sin()) * speed;

    let sprite_x_dir = Vec2::new(sprite_angle.cos(), sprite_angle.sin());
    let obb = OrientedBox {
        axis0: Vec2::new(GIFTSPRITE_HALF_EXTENT_X * scale, 0.0).rotate(sprite_x_dir),
        axis1: Vec2::new(0.0, GIFTSPRITE_HALF_EXTENT_Y * scale).rotate(sprite_x_dir),
    };

    commands.spawn(GifteroidBundle {
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
        despawner: DespawnOnStateEnter(GameState::Game),
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
    mut destroyed_events: EventWriter<GifteroidDestroyedEvent>,
) {
    for (entity_gifteroid, transform_gifteroid, obb, size) in &query_gifteroids {
        let position_gifteroid = transform_gifteroid.translation.truncate();
        for (entity_snowball, transform_snowball) in &query_snowballs {
            // snowballs have a radius, but we ignore it here since they move fast enough
            if point_in_obb(
                obb,
                transform_snowball.translation.truncate(),
                position_gifteroid,
            ) {
                log_gifteroid_despawn(entity_gifteroid);
                commands.entity(entity_gifteroid).despawn();
                commands.entity(entity_snowball).despawn();

                destroyed_events.send(GifteroidDestroyedEvent {
                    split_in_two: matches!(size, GifteroidSize::Small),
                });

                let new_size = match size {
                    GifteroidSize::Large => GifteroidSize::Medium,
                    GifteroidSize::Medium => GifteroidSize::Small,
                    GifteroidSize::Small => continue,
                };

                let seed = query_gifteroids.iter().len();
                let mut rng = StdRng::seed_from_u64(seed as u64);

                for _ in 0..2 {
                    spawn_gifteroid(
                        &mut commands,
                        sprites.gift0.clone(),
                        position_gifteroid,
                        &mut rng,
                        new_size,
                    )
                }
            }
        }
    }
}

fn gifteroid_spaceship_collision(
    query_gifteroids: Query<(&Transform, &OrientedBox), With<GifteroidSize>>,
    mut query_spaceship: Query<(&Transform, &mut SpaceShip)>,
    mut destroyed_events: EventWriter<SpaceShipDestroyedEvent>,
) {
    if query_spaceship.get_single().is_err() {
        return;
    }
    let (spaceship_transform, mut spaceship) = query_spaceship.single_mut();
    if matches!(
        spaceship.state,
        SpaceShipState::Invincible(_) | SpaceShipState::Destroyed
    ) {
        return;
    }
    let (tri_a, tri_b, tri_c) = SpaceShip::bounding_triangle(spaceship_transform);

    // Detect collision by checking line collisions. Not perfect, but good enough and easy to implement
    // outer lines of spaceship
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
            spaceship.state = SpaceShipState::Destroyed;
            destroyed_events.send(SpaceShipDestroyedEvent {
                lives_left_before_destroy: spaceship.lives_left,
            });

            break;
        }
    }
}

fn check_win_condition(
    query_gifteroids: Query<With<GifteroidSize>>,
    mut state: ResMut<State<GameState>>,
) {
    if query_gifteroids.is_empty() {
        state.set(GameState::Highscore).unwrap();
    }
}

fn log_gifteroid_despawn(entity: Entity) {
    #[cfg(feature = "rerun")]
    {
        if let Some(rec) = rerun::RecordingStream::global(rerun::RecordingType::Data) {
            rerun::MsgSender::new("log")
                .with_timepoint(crate::rerun_time())
                .with_component(&[rerun::components::TextEntry::new(
                    format!("despawn {:?}", entity),
                    Some("INFO".to_owned()),
                )])
                .unwrap()
                .send(&rec)
                .map_err(anyhow::Error::msg)
                .ok();
        }
    }
}

#[cfg(feature = "rerun")]
fn send_collision_geom_to_rerun(
    query_spaceship: Query<&Transform, With<SpaceShip>>,
    query_gifteroids: Query<(&Transform, &OrientedBox, &GifteroidSize, Entity)>,
    query_snowballs: Query<&Transform, With<Snowball>>,
) {
    let Some(rec) = rerun::RecordingStream::global(rerun::RecordingType::Data) else {
        return;
    };
    let time = crate::rerun_time();

    use rerun::external::re_log_types::external::{arrow2, arrow2_convert};

    #[derive(arrow2_convert::ArrowField, arrow2_convert::ArrowSerialize)]
    #[arrow_field(transparent)]
    struct GifteroidSizeInt(i32);

    impl rerun::Component for GifteroidSizeInt {
        fn name() -> rerun::ComponentName {
            "ext.gifteroids_size".into()
        }
    }

    rec.record_path_op(
        time.clone(),
        rerun::log::PathOp::clear(true, "collision/gifteroids".into()),
    );

    for (transform_gifteroid, obb, size, entity) in &query_gifteroids {
        let position_gifteroid = transform_gifteroid.translation.truncate();

        // outer lines of gifteroid
        let top_right = position_gifteroid + obb.axis0 + obb.axis1;
        let top_left = position_gifteroid - obb.axis0 + obb.axis1;
        let bottom_left = position_gifteroid - obb.axis0 - obb.axis1;
        let bottom_right = position_gifteroid + obb.axis0 - obb.axis1;

        // TODO: Rerun doesn't yet have support for rotated rectangles.
        let mut lines: Vec<rerun::components::Vec2D> = Vec::new();
        lines.push(top_right.to_array().into());
        lines.push(top_left.to_array().into());
        lines.push(bottom_left.to_array().into());
        lines.push(bottom_right.to_array().into());
        lines.push(top_right.to_array().into());
        rerun::MsgSender::new(format!(
            "collision/gifteroids/{}_{}",
            entity.index(),
            entity.generation()
        ))
        .with_timepoint(time.clone())
        .with_component(&[rerun::components::LineStrip2D(lines)])
        .unwrap()
        .with_component(&[GifteroidSizeInt(*size as i32)])
        .unwrap()
        .send(&rec)
        .map_err(anyhow::Error::msg)
        .ok();
    }

    let points = query_snowballs
        .iter()
        .map(|transform| transform.translation.truncate().to_array().into())
        .collect::<Vec<rerun::components::Point2D>>();
    rerun::MsgSender::new(format!("collision/snowballs"))
        .with_timepoint(time.clone())
        .with_component(&points)
        .unwrap()
        .send(&rec)
        .map_err(anyhow::Error::msg)
        .ok();

    if let Ok(spaceship_transform) = query_spaceship.get_single() {
        let (tri_a, tri_b, tri_c) = SpaceShip::bounding_triangle(spaceship_transform);
        // TODO: Rerun doesn't support triangles yet.
        let mut lines: Vec<rerun::components::Vec2D> = Vec::new();
        lines.push(tri_a.to_array().into());
        lines.push(tri_b.to_array().into());
        lines.push(tri_c.to_array().into());
        lines.push(tri_a.to_array().into());
        rerun::MsgSender::new("collision/gifteroids/grinch")
            .with_timepoint(time.clone())
            .with_component(&[rerun::components::LineStrip2D(lines)])
            .unwrap()
            .send(&rec)
            .map_err(anyhow::Error::msg)
            .ok();
    }
}
