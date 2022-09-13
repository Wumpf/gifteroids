use bevy::{prelude::*, time::FixedTimestep};
use rand::{rngs::StdRng, Rng, SeedableRng};

// Defines the amount of time that should elapse between each physics step.
// A little bit opinionated ;)
const TIME_STEP: f32 = 1.0 / 120.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(control_spaceship)
                .with_system(move_objects)
                .with_system(update_aabb)
                .with_system(wrap_objects_around_screen),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
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
        .insert(MovementSpeed(Vec2::ZERO))
        .insert(AABB::default())
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            transform: Transform {
                scale: Vec3::new(0.5, 0.5, 1.0),
                ..default()
            },
            ..default()
        });

    // The gifteroids
    const GIFTEROIDS_SPAWN_COUNT: u32 = 8;
    const GIFTEROIDS_SPAWN_PLAYER_CLEARANCE: f32 = 80.0;
    const GIFTEROIDS_BASE_SPEED: f32 = 50.0;

    let mut rng = StdRng::seed_from_u64(123);
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
        let movement_orientation = rng.gen_range(0.0..std::f32::consts::TAU);
        let movement = Vec2::new(movement_orientation.sin(), movement_orientation.cos())
            * GIFTEROIDS_BASE_SPEED;

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
            aabb: AABB::default(),
        });
    }
}

#[derive(Component)]
struct SpaceShip;
#[derive(Component)]
struct MovementSpeed(Vec2);
#[derive(Component, Default)]
struct AABB {
    min: Vec2,
    max: Vec2,
}

#[derive(Bundle)]
struct GifteroidBundle {
    size: GifteroidSize,
    #[bundle]
    sprite: SpriteBundle,
    movement: MovementSpeed,
    aabb: AABB,
}

#[derive(Component)]
enum GifteroidSize {
    Large,
    Medium,
    Small,
}

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

fn update_aabb(
    images: Res<Assets<Image>>,
    mut query: Query<(&Transform, &mut AABB, &Handle<Image>)>,
) {
    for (transform, mut aabb, image_handle) in &mut query {
        let image_size = images.get(image_handle).unwrap().size();
        let scaled_size_half = image_size * transform.scale.truncate() * 0.5;
        let pos2d = transform.translation.truncate();
        // TODO: Rotation
        aabb.min = pos2d - scaled_size_half;
        aabb.max = pos2d + scaled_size_half;
    }
}

fn wrap_objects_around_screen(
    camera_query: Query<&OrthographicProjection, With<Camera2d>>,
    mut query: Query<(&mut Transform, &AABB)>,
) {
    let camera = camera_query.single();

    for (mut transform, aabb) in &mut query {
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

fn control_spaceship(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut MovementSpeed, &mut Transform), With<SpaceShip>>,
) {
    const ACCELERATION: f32 = 400.0;
    const ROTATION_SPEED: f32 = 2.0;
    const FRICTION: f32 = 0.5;

    let (mut speed, mut transform) = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        transform.rotate_z(ROTATION_SPEED * TIME_STEP);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        transform.rotate_z(-ROTATION_SPEED * TIME_STEP);
    }
    if keyboard_input.pressed(KeyCode::Up) {
        speed.0 += transform.rotation.mul_vec3(Vec3::Y).truncate() * (ACCELERATION * TIME_STEP);
    }

    speed.0 *= FRICTION.powf(TIME_STEP);
}
