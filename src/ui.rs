use bevy::prelude::*;

use crate::{
    score::Score,
    spaceship::{SpaceShipDestroyedEvent, NUM_LIVES_ON_STARTUP, SPACESHIP_SPRITE_FILE},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_life_display)
            .add_startup_system(setup_score_display)
            .add_system(on_space_ship_destroy)
            .add_system(score_display);
    }
}

#[derive(Component)]
struct SpaceShipLiveDisplay {
    life_icons: Vec<Entity>,
}

#[derive(Component)]
struct ScoreDisplay;

fn setup_life_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let space_ship_image = UiImage::from(asset_server.load(SPACESHIP_SPRITE_FILE));

    let mut life_icons = Vec::new();
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexEnd,
                align_self: AlignSelf::FlexEnd,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            life_icons = (0..NUM_LIVES_ON_STARTUP)
                .map(|_| {
                    parent
                        .spawn_bundle(ImageBundle {
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

    commands.spawn().insert(SpaceShipLiveDisplay { life_icons });
}

fn setup_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(
            TextBundle::from_section(
                "999999",
                TextStyle {
                    font: asset_server.load("Ubuntu-Regular.ttf"),
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
        .insert(ScoreDisplay);
}

fn on_space_ship_destroy(
    mut commands: Commands,
    mut destroyed_events: EventReader<SpaceShipDestroyedEvent>,
    mut query: Query<&mut SpaceShipLiveDisplay>,
) {
    for _ in destroyed_events.iter() {
        let display = &mut query.single_mut();
        if let Some(entity) = display.life_icons.pop() {
            commands.entity(entity).despawn_recursive(); // TODO: despawn is the wrong strategy. want to be able to reset them
        }
    }
}

fn score_display(score: Res<Score>, mut text_query: Query<&mut Text, With<ScoreDisplay>>) {
    let mut text = text_query.single_mut();
    text.sections[0].value = score.0.to_string();
}
