use bevy::{prelude::*, time::FixedTimestep};

// Defines the amount of time that should elapse between each physics step.
// A little bit opinionated ;)
const TIME_STEP: f32 = 1.0 / 120.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup_window)
        .add_startup_system(setup_world)
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

fn setup_window(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_resizable(false);
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

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
    const ACCELERATION: f32 = 100.0;
    const ROTATION_SPEED: f32 = 2.0;

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
}
