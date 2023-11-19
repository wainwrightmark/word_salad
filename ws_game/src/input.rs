use crate::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use strum::EnumIs;
use ws_core::layout::entities::*;
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
        current_level: &mut ResMut<CurrentLevel> ,
        menu_state: &mut ResMut<MenuState>,
        chosen_state: &mut ResMut<ChosenState>,
        input_state: &mut Local<GridInputState>,
        found_words: &mut ResMut<FoundWordsState>,
    ) {
        let level_complete = found_words.is_level_complete(&current_level);
        match self {
            InputType::Start(position) => {
                let Some(layout_entity) = size.try_pick::<GameLayoutEntity>(*position, &()) else {
                    return;
                };
                match layout_entity {
                    GameLayoutEntity::TopBar => {
                        let Some(button) = size.try_pick::<LayoutTopBarButton>(*position, &()) else {
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

                        if level_complete{
                            let Some(entity) = size.try_pick::<CongratsLayoutEntity>(*position, &()) else{return;};

                            match entity{
                                CongratsLayoutEntity::ShareButton => {
                                    //todo wasm share
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        crate::wasm::share();
                                    }
                                },
                                CongratsLayoutEntity::NextButton => {
                                    current_level.to_next_level(found_words.as_mut());
                                },
                            }

                        }
                        else{
                            let Some(tile) = size.try_pick::<LayoutGridTile>(*position, &()) else {return;};
                            let grid = &current_level.level().grid;
                            input_state.handle_input_start(chosen_state, tile.0, grid, found_words);
                        }
                    }
                    GameLayoutEntity::WordList => {
                        let words = &current_level.level().words;
                        let Some(word) = size.try_pick::<LayoutWordTile>(*position, &words) else {
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
                    if level_complete{return;};
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
) {
    if !menu_state.is_closed() {
        return;
    }

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
) {
    if !menu_state.is_closed() {
        return;
    }

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
