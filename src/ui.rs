use bevy::prelude::*;

use crate::{
    score::Score,
    spaceship::{SpaceShipDestroyedEvent, NUM_LIVES_ON_STARTUP, SPACESHIP_SPRITE_FILE},
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
        });

    commands
        .spawn(SpaceShipLiveDisplay { life_icons })
        .insert(DespawnOnStateEnter(GameState::Any));
}

fn setup_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(
            TextBundle::from_section(
                "999999",
                TextStyle {
                    font: asset_server.load("Ubuntu-Regular.ttf"), // TODO: use font resource
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
