use bevy::prelude::*;

mod collision;
mod debug_lines;
mod gifteroids;
mod score;
mod spaceship;
mod ui;

// enum GameState {
//     GameOver,
// }

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_plugin(gifteroids::GifteroidsPlugin)
        .add_plugin(spaceship::SpaceshipPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(score::ScorePlugin)
        .add_system(move_objects);

    #[cfg(feature = "debug_lines")]
    app.add_plugin(debug_lines::DebugLinesPlugin);

    app.run();
}

fn setup(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    // We use the entire screen for the game and don't handle after the fact.
    // Therefore, we resizing while the game is running is not supported.
    window.set_resizable(false);

    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
pub struct MovementSpeed(pub Vec2);

fn move_objects(time: Res<Time>, mut query: Query<(&mut Transform, &MovementSpeed)>) {
    for (mut transform, speed) in query.iter_mut() {
        transform.translation += Vec3::from((speed.0 * time.delta_seconds(), 0.0));
    }
}
