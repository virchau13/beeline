use crate::{
    ui::{spawn_back_button, GameFont},
    world::{GameWorld, LEVELS},
    AppState,
};
use bevy::prelude::*;
use std::{ffi::OsStr, fs, io, path::PathBuf};

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::LevelSelect).with_system(create_level_select),
        )
        .add_system_set(
            SystemSet::on_update(AppState::LevelSelect).with_system(manage_level_select_buttons),
        );
    }
}

#[derive(Component)]
struct LevelSelectButton {
    level: usize,
}

fn create_level_select(mut commands: Commands, font: Res<GameFont>) {
    commands.spawn_bundle(UiCameraBundle::default());

    spawn_back_button(&mut commands, font.get_handle());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                ..Style::default()
            },
            color: Color::NONE.into(),
            ..NodeBundle::default()
        })
        .with_children(|parent| {
            const LEVEL_SELECT_HEIGHT: f32 = 50.0;
            // Spawn level select title
            parent.spawn_bundle(TextBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: Rect {
                        top: Val::Percent(2.0),
                        ..Rect::default()
                    },
                    ..Style::default()
                },
                text: Text::with_section(
                    "Level Select",
                    TextStyle {
                        font: font.get_handle(),
                        font_size: 70.0,
                        ..TextStyle::default()
                    },
                    TextAlignment::default(),
                ),
                ..TextBundle::default()
            });

            // Spawn level selector
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(60.0), Val::Percent(LEVEL_SELECT_HEIGHT)),
                        margin: Rect::all(Val::Auto),
                        flex_direction: FlexDirection::ColumnReverse,
                        ..Style::default()
                    },
                    color: Color::GRAY.into(),
                    ..NodeBundle::default()
                })
                .with_children(|parent| {
                    // Set button height dynamically (based on number of levels)
                    let button_height = LEVEL_SELECT_HEIGHT / LEVELS.len() as f32;

                    for (level, (name, _)) in LEVELS.iter().enumerate() {
                        parent
                            .spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Percent(80.0),
                                        Val::Percent(button_height),
                                    ),
                                    margin: Rect::all(Val::Auto),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Style::default()
                                },
                                ..ButtonBundle::default()
                            })
                            .insert(LevelSelectButton {
                                level,
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(TextBundle {
                                    text: Text::with_section(
                                        *name,
                                        TextStyle {
                                            font: font.get_handle(),
                                            font_size: 30.0,
                                            color: Color::BLACK,
                                        },
                                        TextAlignment::default(),
                                    ),
                                    ..TextBundle::default()
                                });
                            });
                    }
                });
        });
}

fn manage_level_select_buttons(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    interaction: Query<(&Interaction, &LevelSelectButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, level_select_button) in interaction.iter() {
        // Check if the button has been clicked
        if matches!(interaction, Interaction::Clicked) {
            commands.insert_resource(
                GameWorld::load_level(level_select_button.level)
                    .unwrap(),
            );
            state.set(AppState::Game).unwrap();
        }
    }
}
