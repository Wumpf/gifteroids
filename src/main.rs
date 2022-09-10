use bevy::{prelude::*, sprite::MaterialMesh2dBundle, time::FixedTimestep};

// Defines the amount of time that should elapse between each physics step.
// A little bit opinionated ;)
const TIME_STEP: f32 = 1.0 / 120.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(control_spaceship)
                .with_system(move_objects),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn()
        .insert(SpaceShip)
        .insert(MovementSpeed(Vec2::ZERO))
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(64.)),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            ..default()
        });
}

#[derive(Component)]
struct SpaceShip;

#[derive(Component)]
struct MovementSpeed(Vec2);

fn move_objects(mut query: Query<(&mut Transform, &MovementSpeed)>) {
    for (mut transform, speed) in &mut query {
        transform.translation += Vec3::from((speed.0 * TIME_STEP, 0.0));
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
