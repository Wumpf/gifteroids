use bevy::{prelude::*, time::FixedTimestep};
use rand::{random, rngs::StdRng, Rng, SeedableRng};

#[cfg(feature = "debug_lines")]
use bevy_prototype_debug_lines::*;

// Defines the amount of time that should elapse between each physics step.
// A little bit opinionated ;)
const TIME_STEP: f32 = 1.0 / 120.0;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(control_spaceship)
                .with_system(move_objects)
                .with_system(screen_wrap_obb_entities)
                .with_system(screen_wrap_snowball),
        )
        .add_system(shoot_snowballs);

    #[cfg(feature = "debug_lines")]
    app.add_plugin(DebugLinesPlugin::default())
        .add_system(draw_obb_debug_lines)
        .add_system(draw_spaceship_debug_lines);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    // We use the entire screen for the game and don't handle after the fact.
    // Therefore, we resizing while the game is running is not supported.
    window.set_resizable(false);
    let screen_size = Vec2 {
        x: window.width(),
        y: window.height(),
    };

    commands.spawn_bundle(Camera2dBundle::default());

    // The player
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

    spawn_gifteroids(screen_size, asset_server, commands);
}

fn spawn_gifteroids(screen_size: Vec2, asset_server: Res<AssetServer>, mut commands: Commands) {
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

#[derive(Component)]
struct SpaceShip;
#[derive(Component)]
struct SnowballShootingCooldown(f32);

#[derive(Component)]
struct MovementSpeed(Vec2);
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

#[derive(Component)]
struct OrientedBox {
    // Half axis from center. axis are in a right angle
    axis0: Vec2,
    axis1: Vec2,
}

struct AABB {
    min: Vec2,
    max: Vec2,
}

#[derive(Component)]
struct Snowball;

struct SnowballSprite(Handle<Image>);

impl AABB {
    fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

fn move_objects(mut query: Query<(&mut Transform, &MovementSpeed)>) {
    for (mut transform, speed) in &mut query {
        transform.translation += Vec3::from((speed.0 * TIME_STEP, 0.0));
    }
}

#[cfg(feature = "debug_lines")]
fn draw_obb_debug_lines(
    mut lines: ResMut<DebugLines>,
    mut query: Query<(&mut Transform, &OrientedBox)>,
) {
    for (transform, obb) in &mut query {
        lines.line_colored(
            transform.translation - Vec3::from((obb.axis0 - obb.axis1, 0.0)),
            transform.translation - Vec3::from((obb.axis0 + obb.axis1, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation + Vec3::from((obb.axis0 - obb.axis1, 0.0)),
            transform.translation + Vec3::from((obb.axis0 + obb.axis1, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation - Vec3::from((obb.axis1 - obb.axis0, 0.0)),
            transform.translation - Vec3::from((obb.axis1 + obb.axis0, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
        lines.line_colored(
            transform.translation + Vec3::from((obb.axis1 - obb.axis0, 0.0)),
            transform.translation + Vec3::from((obb.axis1 + obb.axis0, 0.0)),
            0.0,
            Color::ORANGE_RED,
        );
    }
}

#[cfg(feature = "debug_lines")]
fn draw_spaceship_debug_lines(
    mut lines: ResMut<DebugLines>,
    query: Query<&Transform, With<SpaceShip>>,
) {
    for transform in &query {
        let (tri_a, tri_b, tri_c) = span_spaceship_triangle(transform);

        lines.line_colored(tri_a.extend(0.0), tri_b.extend(0.0), 0.0, Color::ORANGE_RED);
        lines.line_colored(tri_b.extend(0.0), tri_c.extend(0.0), 0.0, Color::ORANGE_RED);
        lines.line_colored(tri_c.extend(0.0), tri_a.extend(0.0), 0.0, Color::ORANGE_RED);
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
        let aabb = AABB {
            min: transform.translation.truncate() - max_axis_step,
            max: transform.translation.truncate() + max_axis_step,
        };

        if aabb.max.y < camera.bottom {
            transform.translation.y = camera.top + aabb.size().y * 0.5 - 0.1;
        } else if aabb.min.y > camera.top {
            transform.translation.y = camera.bottom - aabb.size().y * 0.5 + 0.1;
        }
        if aabb.max.x < camera.left {
            transform.translation.x = camera.right + aabb.size().x * 0.5 - 0.1;
        } else if aabb.min.x > camera.right {
            transform.translation.x = camera.left - aabb.size().x * 0.5 + 0.1;
        }
    }
}

fn screen_wrap_snowball(
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

fn span_spaceship_triangle(transform: &Transform) -> (Vec2, Vec2, Vec2) {
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

fn control_spaceship(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut MovementSpeed, &mut Transform), With<SpaceShip>>,
) {
    const ACCELERATION: f32 = 400.0;
    const ROTATION_SPEED: f32 = 2.0;
    const FRICTION: f32 = 0.5;

    let (mut speed, mut transform) = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        transform.rotate_z(ROTATION_SPEED * TIME_STEP);
        transform.rotation = transform.rotation.normalize();
    }
    if keyboard_input.pressed(KeyCode::Right) {
        transform.rotate_z(-ROTATION_SPEED * TIME_STEP);
        transform.rotation = transform.rotation.normalize();
    }
    if keyboard_input.pressed(KeyCode::Up) {
        speed.0 += transform.rotation.mul_vec3(Vec3::Y).truncate() * (ACCELERATION * TIME_STEP);
    }
    speed.0 *= FRICTION.powf(TIME_STEP);

    let camera = camera_query.single();
    screen_wrap_space_ship(&mut transform, camera);
}

fn screen_wrap_space_ship(transform: &mut Transform, camera: &OrthographicProjection) {
    let (tri_a, tri_b, tri_c) = span_spaceship_triangle(transform);

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

fn shoot_snowballs(
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

    let (tri_a, _, _) = span_spaceship_triangle(transform);

    cooldown.0 += SNOWBALL_COOLDOWN_SECONDS;
    commands
        .spawn()
        .insert(Snowball)
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
