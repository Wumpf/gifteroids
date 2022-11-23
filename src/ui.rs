use std::sync::Arc;

use bevy::prelude::*;
use parking_lot::Mutex;

use crate::{
    score::Score,
    spaceship::{SpaceShipDestroyedEvent, NUM_LIVES_ON_STARTUP, SPACESHIP_SPRITE_FILE},
    web_request::{publish_score, query_highscore},
    DespawnOnStateEnter, GameState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(
                SystemSet::on_enter(GameState::Game)
                    .with_system(setup_life_display)
                    .with_system(setup_score_display),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(on_space_ship_destroy)
                    .with_system(score_display),
            )
            .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(show_game_over))
            .add_system_set(
                SystemSet::on_update(GameState::GameOver).with_system(start_game_on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(GameState::Highscore).with_system(setup_highscore_screen),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Highscore)
                    .with_system(start_game_on_enter)
                    .with_system(check_score_query),
            );
    }
}

#[derive(Component)]
struct SpaceShipLiveDisplay {
    life_icons: Vec<Entity>,
}

#[derive(Resource)]
struct Fonts {
    font: Handle<Font>,
}

impl Fonts {
    fn text(&self, text: impl Into<String>, font_size: f32) -> TextBundle {
        TextBundle::from_section(
            text,
            TextStyle {
                font: self.font.clone(),
                font_size,
                color: Color::WHITE,
            },
        )
    }
}

const BACKGROUND_COLOR: BackgroundColor = BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.5));

#[derive(Component)]
struct ScoreDisplay;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Fonts {
        font: asset_server.load("Ubuntu-Regular.ttf"),
    });
}

fn setup_life_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let space_ship_image = UiImage::from(asset_server.load(SPACESHIP_SPRITE_FILE));

    let mut life_icons = Vec::new();
    commands
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::FlexStart,
                align_self: AlignSelf::FlexStart,
                ..default()
            },
            background_color: BACKGROUND_COLOR,
            ..default()
        })
        .with_children(|parent| {
            life_icons = (0..NUM_LIVES_ON_STARTUP)
                .map(|_| {
                    parent
                        .spawn(ImageBundle {
                            style: Style {
                                size: Size::new(Val::Px(60.0), Val::Auto),
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            image: space_ship_image.clone(),
                            ..default()
                        })
                        .id()
                })
                .collect();
        })
        .insert(DespawnOnStateEnter(GameState::Any));

    commands
        .spawn(SpaceShipLiveDisplay { life_icons })
        .insert(DespawnOnStateEnter(GameState::Any));
}

fn setup_score_display(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn(
            TextBundle::from_section(
                "999999",
                TextStyle {
                    font: fonts.font.clone(),
                    font_size: 80.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_CENTER)
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ScoreDisplay)
        .insert(DespawnOnStateEnter(GameState::Any));
}

fn on_space_ship_destroy(
    mut commands: Commands,
    mut destroyed_events: EventReader<SpaceShipDestroyedEvent>,
    mut life_display: Query<&mut SpaceShipLiveDisplay>,
    mut state: ResMut<State<GameState>>,
) {
    let Some(destroyed_event) = destroyed_events.iter().next() else {
        return;
    };

    let display = &mut life_display.single_mut();

    if destroyed_event.lives_left_before_destroy == 0 {
        state.set(GameState::GameOver).unwrap();
    } else {
        debug_assert_eq!(
            destroyed_event.lives_left_before_destroy,
            display.life_icons.len() as u32
        );
        if let Some(entity) = display.life_icons.pop() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn score_display(score: Res<Score>, mut text_query: Query<&mut Text, With<ScoreDisplay>>) {
    let mut text = text_query.single_mut();
    text.sections[0].value = score.0.to_string();
}

fn start_game_on_enter(keys: Res<Input<KeyCode>>, mut game_state: ResMut<State<GameState>>) {
    if keys.pressed(KeyCode::NumpadEnter) || keys.pressed(KeyCode::Return) {
        game_state.overwrite_set(GameState::Game).unwrap();
    }
}

fn show_game_over(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: BACKGROUND_COLOR,
            ..default()
        })
        .insert(DespawnOnStateEnter(GameState::Any))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Game Over",
                TextStyle {
                    font: fonts.font.clone(),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
            ));
            parent.spawn(TextBundle::from_section(
                "Press Enter to try again",
                TextStyle {
                    font: fonts.font.clone(),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
            ));
        });
}

enum HighscorePublishAndQueryResult {
    Pending,
    FailedToPublish(String),
    FailedToQuery(String),
    Success(String),
}

#[derive(Resource)]
struct HighscoreDisplay {
    processed_query_result: bool,
    highscore_publish_and_query_result: Arc<Mutex<HighscorePublishAndQueryResult>>,
    ui_entity: Entity,
}

fn setup_highscore_screen(mut commands: Commands, fonts: Res<Fonts>, score: Res<Score>) {
    let ui_entity = spawn_highscore_ui(
        &mut commands,
        &fonts,
        &score,
        &HighscorePublishAndQueryResult::Pending,
    );

    let highscore_display = HighscoreDisplay {
        processed_query_result: false,
        highscore_publish_and_query_result: Arc::new(Mutex::new(
            HighscorePublishAndQueryResult::Pending,
        )),
        ui_entity,
    };
    {
        let query_result = highscore_display.highscore_publish_and_query_result.clone();
        publish_score(score.0, move |result| {
            match result {
                Ok(response) => {
                    if response.status != 200 {
                        *query_result.lock() =
                            HighscorePublishAndQueryResult::FailedToPublish(response.status_text);
                        return;
                    }
                    query_highscore(move |result| match result {
                        Ok(response) => {
                            *query_result.lock() = HighscorePublishAndQueryResult::Success(
                                String::from_utf8(response.bytes).unwrap(),
                            );
                        }
                        Err(err) => {
                            *query_result.lock() =
                                HighscorePublishAndQueryResult::FailedToQuery(err);
                        }
                    });
                }
                Err(err) => {
                    *query_result.lock() = HighscorePublishAndQueryResult::FailedToPublish(err);
                }
            };
        });
    }
    commands.insert_resource(highscore_display);
}

fn spawn_highscore_ui(
    commands: &mut Commands,
    fonts: &Fonts,
    score: &Score,
    score_query: &HighscorePublishAndQueryResult,
) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: BACKGROUND_COLOR,
            ..default()
        })
        .insert(DespawnOnStateEnter(GameState::Any))
        .with_children(|parent| {
            parent.spawn(fonts.text("You prevented christmas!", 100.0));
            parent.spawn(fonts.text(format!("Your score was {}", score.0), 40.0));
            parent.spawn(fonts.text("High Score", 60.0));

            match score_query {
                HighscorePublishAndQueryResult::Pending => {
                    parent.spawn(fonts.text("... Submitting Highscore ...", 40.0));
                }
                HighscorePublishAndQueryResult::FailedToPublish(err) => {
                    parent.spawn(fonts.text("Failed to publish highscore:", 25.0));
                    parent.spawn(fonts.text(err, 25.0));
                }
                HighscorePublishAndQueryResult::FailedToQuery(err) => {
                    parent.spawn(fonts.text("Failed to query highscore:", 25.0));
                    parent.spawn(fonts.text(err, 25.0));
                }
                HighscorePublishAndQueryResult::Success(score_response) => {
                    // oh god. a json parser. for this?!
                    // But handling all corner cases of json unicode characters is not something I want to handle here...
                    match score_response.parse::<tinyjson::JsonValue>() {
                        Ok(json) => {
                            if let tinyjson::JsonValue::Object(highscore) = json {
                                let mut scores = highscore
                                    .iter()
                                    .take(10)
                                    .map(|(key, value)| {
                                        (
                                            key,
                                            match value {
                                                tinyjson::JsonValue::Number(i) => *i as u32,
                                                _ => 0, // eh whatever. it's late.
                                            },
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                scores.sort_by(|(_, score0), (_, score1)| score1.cmp(score0));

                                parent
                                    .spawn(NodeBundle {
                                        style: Style {
                                            flex_direction: FlexDirection::Row,
                                            ..default()
                                        },
                                        ..default()
                                    })
                                    .add_children(|parent| {
                                        spawn_score_column(
                                            fonts,
                                            parent,
                                            (1..(scores.len() + 1)).map(|i| format!("{i}.")),
                                        );
                                        spawn_score_column(
                                            fonts,
                                            parent,
                                            scores.iter().map(|s| s.0.clone()),
                                        );
                                        spawn_score_column(
                                            fonts,
                                            parent,
                                            scores.iter().map(|s| s.1.to_string()),
                                        );
                                    });
                            } else {
                                parent.spawn(
                                    fonts.text(
                                        "Highscore was not a json dictionary as expected",
                                        25.0,
                                    ),
                                );
                            }
                        }
                        Err(err) => {
                            parent.spawn(fonts.text("Failed to parse highscore json:", 25.0));
                            parent.spawn(fonts.text(format!("{err}"), 25.0));
                        }
                    }
                }
            }

            parent.spawn(fonts.text("Press Enter to try again", 40.0));
        })
        .id()
}

fn spawn_score_column(
    fonts: &Fonts,
    parent: &mut ChildBuilder,
    strings: impl Iterator<Item = String>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect {
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                },
                ..default()
            },
            ..default()
        })
        .add_children(|parent| {
            for s in strings {
                parent.spawn(fonts.text(s, 25.0));
            }
        });
}

fn check_score_query(
    mut commands: Commands,
    fonts: Res<Fonts>,
    score: Res<Score>,
    mut score_query: ResMut<HighscoreDisplay>,
) {
    if score_query.processed_query_result {
        return;
    }

    score_query.ui_entity = {
        let result_lock = score_query.highscore_publish_and_query_result.lock();
        let result = &*result_lock;
        if matches!(result, HighscorePublishAndQueryResult::Pending) {
            return;
        }
        commands.entity(score_query.ui_entity).despawn_recursive();
        spawn_highscore_ui(&mut commands, &fonts, &score, result)
    };
    score_query.processed_query_result = true;
}
