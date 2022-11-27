use bevy::{prelude::*, time::FixedTimestep};

use crate::{
    gifteroids::GifteroidDestroyedEvent, santa::SantaDestroyedEvent,
    spaceship::SpaceShipDestroyedEvent, GameState,
};

pub struct ScorePlugin;

const START_SCORE: u32 = 1000;
const SCORE_REDUCTION_FREQUENCY_SECONDS: f64 = 1.0;
const SCORE_REDUCTION: u32 = 10;
const SCORE_PER_GIFTEROID: u32 = 10;
const SCORE_PER_SANTA: u32 = 150;
const SCORE_LOSS_PER_LIFE_LOST: u32 = 100;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(on_asteroid_destroyed)
                    .with_system(on_santa_destroyed)
                    .with_system(on_spaceship_destroyed),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_run_criteria(FixedTimestep::step(SCORE_REDUCTION_FREQUENCY_SECONDS))
                    .with_system(reduce_score),
            );
    }
}

#[derive(Resource)]
pub struct Score(pub u32);

fn setup(mut commands: Commands) {
    commands.insert_resource(Score(START_SCORE));
}

fn reduce_score(mut score: ResMut<Score>) {
    score.0 = score.0.saturating_sub(SCORE_REDUCTION);
}

fn on_asteroid_destroyed(
    mut events: EventReader<GifteroidDestroyedEvent>,
    mut score: ResMut<Score>,
) {
    for _ in events.iter() {
        score.0 += SCORE_PER_GIFTEROID;
    }
}

fn on_santa_destroyed(mut events: EventReader<SantaDestroyedEvent>, mut score: ResMut<Score>) {
    for _ in events.iter() {
        score.0 += SCORE_PER_SANTA;
    }
}

fn on_spaceship_destroyed(
    mut events: EventReader<SpaceShipDestroyedEvent>,
    mut score: ResMut<Score>,
) {
    for _ in events.iter() {
        score.0 = score.0.saturating_sub(SCORE_LOSS_PER_LIFE_LOST);
    }
}
