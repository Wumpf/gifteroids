use bevy::{prelude::*, time::FixedTimestep};

use crate::gifteroids::GifteroidDestroyedEvent;

pub struct ScorePlugin;

const START_SCORE: u32 = 5000;
const SCORE_REDUCTION_FREQUENCY_SECONDS: f64 = 1.0;
const SCORE_REDUCTION: u32 = 50;
const SCORE_PER_GIFTEROID: u32 = 100;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system(on_asteroid_destroyed)
            .add_system_set(
                SystemSet::new()
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
