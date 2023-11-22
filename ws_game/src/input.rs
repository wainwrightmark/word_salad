use crate::prelude::{
    level_group_layout::LevelGroupLayout, levels_menu_layout::LevelsMenuLayoutEntity,
    main_menu_layout::MainMenuLayoutEntity, *,
};
use bevy::{prelude::*, window::PrimaryWindow};
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use strum::EnumIs;
use ws_core::layout::entities::*;
use ws_levels::level_sequence::LevelSequence;
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_mouse_input);
        app.add_systems(Update, handle_touch_input);
    }
}

const MOVE_TOLERANCE: f32 = 0.3;
#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum InputType {
    Start(Vec2),
    Move(Vec2),
    End(Option<Vec2>),
}

impl InputType {
    pub fn handle(
        &self,

        size: &Size,
        current_level: &mut ResMut<CurrentLevel>,
        menu_state: &mut ResMut<MenuState>,
        chosen_state: &mut ResMut<ChosenState>,
        input_state: &mut Local<GridInputState>,
        found_words: &mut ResMut<FoundWordsState>,
        video_state: &VideoResource,
        video_events: &AsyncEventWriter<VideoEvent>,
    ) {
        let level_complete = found_words.is_level_complete();
        match self {
            InputType::Start(position) => {
                if !menu_state.is_closed() {

                    if let Some(button) = size.try_pick::<LayoutTopBarButton>(*position, &()) {
                        match button {
                            LayoutTopBarButton::MenuBurgerButton => {
                                *menu_state.as_mut() = MenuState::ShowMainMenu;
                            }
                            LayoutTopBarButton::TimeCounter => {}
                            LayoutTopBarButton::HintCounter => {}
                        }
                        return;
                    }
                    match menu_state.as_ref() {
                        MenuState::Closed => {}
                        MenuState::ShowMainMenu => {
                            let Some(entity) =
                                size.try_pick::<MainMenuLayoutEntity>(*position, &())
                            else {
                                return;
                            };

                            match entity {
                                MainMenuLayoutEntity::Resume => {
                                    *menu_state.as_mut() = MenuState::Closed;
                                }
                                MainMenuLayoutEntity::ChooseLevel => {
                                    *menu_state.as_mut() = MenuState::ChooseLevelsPage;
                                }
                                MainMenuLayoutEntity::ResetLevel => {
                                    current_level.set_changed();
                                    *found_words.as_mut() =
                                        FoundWordsState::new_from_level(&current_level);
                                    *chosen_state.as_mut() = ChosenState::default();

                                    *menu_state.as_mut() = MenuState::Closed;
                                }
                                MainMenuLayoutEntity::Video => {
                                    video_state.toggle_video_streaming(video_events.clone());
                                    *menu_state.as_mut() = MenuState::Closed;
                                }
                            }
                        }
                        MenuState::ChooseLevelsPage => {
                            let Some(entity) =
                                size.try_pick::<LevelsMenuLayoutEntity>(*position, &())
                            else {
                                return;
                            };

                            match entity {
                                LevelsMenuLayoutEntity::DailyChallenge => {
                                    current_level
                                        .to_level(found_words, LevelSequence::DailyChallenge);
                                    *menu_state.as_mut() = MenuState::Closed;
                                }
                                LevelsMenuLayoutEntity::Tutorial => {
                                    current_level.to_level(found_words, LevelSequence::Tutorial);
                                    *menu_state.as_mut() = MenuState::Closed;
                                }
                                LevelsMenuLayoutEntity::AdditionalLevel(level_group) => {
                                    *menu_state.as_mut() = MenuState::LevelGroupPage(level_group);
                                }
                                LevelsMenuLayoutEntity::Back => {
                                    *menu_state.as_mut() = MenuState::ShowMainMenu;
                                }
                            }
                        }
                        MenuState::LevelGroupPage(group) => {
                            let Some(entity) = size.try_pick::<LevelGroupLayout>(*position, &group)
                            else {
                                return;
                            };

                            let Some(sequence) = group.get_sequences().get(entity.index) else {
                                return;
                            };

                            current_level.to_level(found_words, *sequence);
                            *menu_state.as_mut() = MenuState::Closed;
                        }
                    }
                }

                let Some(layout_entity) = size.try_pick::<GameLayoutEntity>(*position, &()) else {
                    return;
                };

                match layout_entity {
                    GameLayoutEntity::TopBar => {
                        let Some(button) = size.try_pick::<LayoutTopBarButton>(*position, &())
                        else {
                            return;
                        };
                        match button {
                            LayoutTopBarButton::MenuBurgerButton => {
                                *menu_state.as_mut() = MenuState::ShowMainMenu;
                            }
                            LayoutTopBarButton::TimeCounter => {}
                            LayoutTopBarButton::HintCounter => {
                                found_words.try_hint(current_level);
                            }
                        }
                    }
                    GameLayoutEntity::TextArea => {}
                    GameLayoutEntity::Grid => {
                        if level_complete {
                            let Some(entity) =
                                size.try_pick::<CongratsLayoutEntity>(*position, &())
                            else {
                                return;
                            };

                            match entity {
                                CongratsLayoutEntity::ShareButton => {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        crate::wasm::share();
                                    }
                                }
                                CongratsLayoutEntity::NextButton => {
                                    current_level.to_next_level(found_words.as_mut());
                                }
                                CongratsLayoutEntity::LevelTime => {}
                                CongratsLayoutEntity::HintsUsed => {}
                            }
                        } else {
                            let Some(tile) = size.try_pick::<LayoutGridTile>(*position, &()) else {
                                return;
                            };
                            let grid = &current_level.level().grid;
                            input_state.handle_input_start(chosen_state, tile.0, grid, found_words);
                        }
                    }
                    GameLayoutEntity::WordList => {
                        let words = &current_level.level().words;
                        let Some(word) = size.try_pick::<LayoutWordTile>(*position, words) else {
                            return;
                        };

                        found_words.try_hint_word(current_level, word.0);
                    }
                }
            }
            InputType::Move(position) => {
                if level_complete {
                    return;
                };
                let Some(tile) =
                    size.try_pick_with_tolerance::<LayoutGridTile>(*position, MOVE_TOLERANCE, &())
                else {
                    return;
                };
                let grid = &current_level.level().grid;
                input_state.handle_input_move(chosen_state, tile.0, grid, found_words);
            }
            InputType::End(position) => match position {
                Some(position) => {
                    if level_complete {
                        return;
                    };
                    if let Some(tile) = size.try_pick::<LayoutGridTile>(*position, &()) {
                        input_state.handle_input_end(chosen_state, tile.0);
                    } else {
                        input_state.handle_input_end_no_location();
                    }
                }
                None => input_state.handle_input_end_no_location(),
            },
        }
    }
}

fn handle_mouse_input(
    mouse_input: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    size: Res<Size>,
    mut current_level: ResMut<CurrentLevel>,
    mut menu_state: ResMut<MenuState>,
    mut chosen_state: ResMut<ChosenState>,
    mut input_state: Local<GridInputState>,
    mut found_words: ResMut<FoundWordsState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>,
) {
    let input_type = if mouse_input.just_released(MouseButton::Left) {
        let position_option = get_cursor_position(q_windows);
        InputType::End(position_option)
    } else if mouse_input.just_pressed(MouseButton::Left) {
        let Some(position) = get_cursor_position(q_windows) else {
            return;
        };
        InputType::Start(position)
    } else if mouse_input.pressed(MouseButton::Left) {
        let Some(position) = get_cursor_position(q_windows) else {
            return;
        };
        InputType::Move(position)
    } else {
        return;
    };

    input_type.handle(
        &size,
        &mut current_level,
        &mut menu_state,
        &mut chosen_state,
        &mut input_state,
        &mut found_words,
        &video_state,
        &video_events,
    );
}

fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    size: Res<Size>,

    mut current_level: ResMut<CurrentLevel>,
    mut menu_state: ResMut<MenuState>,
    mut chosen_state: ResMut<ChosenState>,
    mut input_state: Local<GridInputState>,
    mut found_words: ResMut<FoundWordsState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>,
) {
    for ev in touch_events.read() {
        let input_type = match ev.phase {
            bevy::input::touch::TouchPhase::Started => {
                let Some(position) = get_touch_position(ev.position, &q_camera, &size) else {
                    continue;
                };
                InputType::Start(position)
            }
            bevy::input::touch::TouchPhase::Moved => {
                let Some(position) = get_touch_position(ev.position, &q_camera, &size) else {
                    continue;
                };
                InputType::Move(position)
            }
            bevy::input::touch::TouchPhase::Ended => {
                let position_option = get_touch_position(ev.position, &q_camera, &size);
                InputType::End(position_option)
            }
            bevy::input::touch::TouchPhase::Canceled => InputType::End(None),
        };

        input_type.handle(
            &size,
            &mut current_level,
            &mut menu_state,
            &mut chosen_state,
            &mut input_state,
            &mut found_words,
            &video_state,
            &video_events,
        );
    }
}

fn get_touch_position(
    position: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
    size: &Size,
    //tolerance: f32,
) -> Option<Vec2> {
    let p = convert_screen_to_world_position(position, q_camera)?;

    let p = Vec2 {
        x: p.x + (size.scaled_width * 0.5),
        y: (size.scaled_height * 0.5) - p.y,
    };
    Some(p)
}

fn convert_screen_to_world_position(
    screen_pos: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    camera.viewport_to_world_2d(camera_transform, screen_pos)
}

fn get_cursor_position(q_windows: Query<&Window, With<PrimaryWindow>>) -> Option<Vec2> {
    let window = q_windows.iter().next()?;
    window.cursor_position()
}
