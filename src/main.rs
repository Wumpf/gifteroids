use bevy::prelude::*;

mod collision;
mod debug_lines;
mod gifteroids;
mod score;
mod spaceship;
mod ui;
mod web_request;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameState {
    Game,
    GameOver,
    Highscore,

    // Not used for states, but useful for DespawnOnStateEnter
    Any,
}

#[derive(Component)]
pub struct DespawnOnStateEnter(GameState);

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_state(GameState::Game)
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup)
        .add_plugin(gifteroids::GifteroidsPlugin)
        .add_plugin(spaceship::SpaceshipPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(score::ScorePlugin)
        .add_system_set(SystemSet::on_enter(GameState::Game).with_system(despawn_on_enter))
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(despawn_on_enter))
        .add_system_set(SystemSet::on_enter(GameState::Highscore).with_system(despawn_on_enter))
        .add_system_set(SystemSet::on_update(GameState::Game).with_system(move_objects))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(move_objects));

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

fn despawn_on_enter(
    mut commands: Commands,
    query: Query<(Entity, &DespawnOnStateEnter)>,
    state: Res<State<GameState>>,
) {
    let state = *state.current();
    for (entity, kill_state) in &query {
        if kill_state.0 == state || kill_state.0 == GameState::Any {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component)]
pub struct MovementSpeed(pub Vec2);

fn move_objects(time: Res<Time>, mut query: Query<(&mut Transform, &MovementSpeed)>) {
    for (mut transform, speed) in query.iter_mut() {
        transform.translation += Vec3::from((speed.0 * time.delta_seconds(), 0.0));
    }
}
