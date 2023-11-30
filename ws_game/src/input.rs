use crate::{
    completion::TotalCompletion,
    prelude::{
        level_group_layout::LevelGroupLayoutEntity, levels_menu_layout::LevelsMenuLayoutEntity,
        main_menu_layout::MainMenuLayoutEntity, *,
    }, menu_layout::main_menu_back_button::MainMenuBackButton,
};
use bevy::{prelude::*, window::PrimaryWindow};
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use strum::EnumIs;
use ws_core::layout::entities::*;
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_mouse_input);
        app.add_systems(Update, handle_touch_input);

        app.add_plugins(ButtonPlugin);
    }
}

const MOVE_TOLERANCE: f32 = 0.3;
#[derive(Debug, Clone, PartialEq, EnumIs)]
pub enum InputType {
    Start(Vec2),
    Move(Vec2),
    End(Option<Vec2>),
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum InteractionEntity {
    Button(ButtonInteraction),
    Tile(Tile),
}

impl InteractionEntity {
    pub fn try_get_button<T: LayoutStructure + Into<ButtonInteraction>>(
        position: &Vec2,
        size: &Size,
        context: &T::Context,
    ) -> Option<Self> {
        return size
            .try_pick::<T>(*position, context)
            .map(|x| InteractionEntity::Button(x.into()));
    }

    pub fn try_find(
        position: &Vec2,
        size: &Size,
        menu_state: &MenuState,
        popup_state: &PopupState,
        current_level: &CurrentLevel,
        is_level_complete: bool,
        grid_tolerance: Option<f32>,
    ) -> Option<Self> {
        match popup_state {
            PopupState::None => {}
            PopupState::BuyMoreHints => {
                return match size.try_pick::<BuyMoreHintsLayoutEntity>(*position, &()) {
                    Some(entity) => match entity {
                        BuyMoreHintsLayoutEntity::Title => None,
                        BuyMoreHintsLayoutEntity::BuyMoreHintsButton => {
                            Some(InteractionEntity::Button(ButtonInteraction::BuyMoreHints))
                        }
                        BuyMoreHintsLayoutEntity::SufferAloneButton => {
                            Some(InteractionEntity::Button(ButtonInteraction::ClosePopups))
                        }
                        BuyMoreHintsLayoutEntity::Box => None,
                    },
                    None => None,
                }
            }
        }

        let tbi = Self::try_get_button::<LayoutTopBarButton>(position, size, &());
        if tbi.is_some() {
            return tbi;
        }


        match menu_state {
            MenuState::Closed => {
                if is_level_complete {
                    return Self::try_get_button::<CongratsLayoutEntity>(position, size, &());
                }

                let Some(layout_entity) = size.try_pick::<GameLayoutEntity>(*position, &()) else {
                    return None;
                };
                match layout_entity {
                    GameLayoutEntity::TopBar => {
                        return Self::try_get_button::<LayoutTopBarButton>(position, size, &());
                    }
                    GameLayoutEntity::TextArea => {
                        return None;
                    }
                    GameLayoutEntity::Grid => match grid_tolerance {
                        Some(tolerance) => {
                            return size
                                .try_pick_with_tolerance::<LayoutGridTile>(
                                    *position,
                                    tolerance,
                                    &(),
                                )
                                .map(|t| Self::Tile(t.0))
                        }
                        None => {
                            return size
                                .try_pick::<LayoutGridTile>(*position, &())
                                .map(|t| Self::Tile(t.0))
                        }
                    },
                    GameLayoutEntity::WordList => {
                        return Self::try_get_button::<LayoutWordTile>(
                            position,
                            size,
                            &current_level.level().words,
                        );
                    }
                }
            }

            MenuState::ShowMainMenu => {
                if let  Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &()){
                    return Some(back);
                }

                return Self::try_get_button::<MainMenuLayoutEntity>(position, size, &());
            }
            MenuState::ChooseLevelsPage => {
                if let  Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &()){
                    return Some(back);
                }

                return Self::try_get_button::<LevelsMenuLayoutEntity>(position, size, &());
            }
            MenuState::LevelGroupPage(group) => {
                if let  Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &()){
                    return Some(back);
                }

                return Self::try_get_button::<LevelGroupLayoutEntity>(position, size, &group);
            }
        }
    }
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
        total_completion: &TotalCompletion,
        pressed_button: &mut ResMut<PressedButton>,
        hint_state: &mut ResMut<HintState>,
        popup_state: &mut ResMut<PopupState>,
        video_state: &VideoResource,
        video_events: &AsyncEventWriter<VideoEvent>,
    ) {
        let is_level_complete = found_words.is_level_complete();

        let button_interaction: Option<ButtonInteraction> = match self {
            InputType::Start(position) => {
                if let Some(interaction) = InteractionEntity::try_find(
                    position,
                    size,
                    menu_state,
                    &popup_state,
                    current_level,
                    is_level_complete,
                    None,
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => {
                            if button.button_press_type().is_on_start() {
                                button.on_pressed(
                                    current_level,
                                    menu_state,
                                    chosen_state,
                                    found_words,
                                    hint_state,
                                    popup_state,
                                    total_completion,
                                    video_state,
                                    video_events,
                                );
                            }

                            Some(button)
                        }
                        InteractionEntity::Tile(tile) => {
                            input_state.handle_input_start(
                                chosen_state,
                                tile,
                                &current_level.level().grid,
                                found_words,
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            }
            InputType::Move(position) => {
                if let Some(interaction) = InteractionEntity::try_find(
                    position,
                    size,
                    menu_state,
                    &popup_state,
                    current_level,
                    is_level_complete,
                    Some(MOVE_TOLERANCE),
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => Some(button),
                        InteractionEntity::Tile(tile) => {
                            input_state.handle_input_move(
                                chosen_state,
                                tile,
                                &current_level.level().grid,
                                found_words,
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            }
            InputType::End(Some(position)) => {
                if let Some(interaction) = InteractionEntity::try_find(
                    position,
                    size,
                    menu_state,
                    &popup_state,
                    current_level,
                    is_level_complete,
                    None,
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => {
                            if button.button_press_type().is_on_end() {
                                button.on_pressed(
                                    current_level,
                                    menu_state,
                                    chosen_state,
                                    found_words,
                                    hint_state,
                                    popup_state,
                                    total_completion,
                                    video_state,
                                    video_events,
                                );
                            }

                            input_state.handle_input_end_no_location();
                            None
                        }
                        InteractionEntity::Tile(tile) => {
                            input_state.handle_input_end(chosen_state, tile);
                            None
                        }
                    }
                } else {
                    input_state.handle_input_end_no_location();
                    None
                }
            }
            InputType::End(None) => {
                input_state.handle_input_end_no_location();
                None
            }
        };

        pressed_button.interaction = button_interaction;
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
    mut pressed_button: ResMut<PressedButton>,
    mut hint_state: ResMut<HintState>,
    mut popup_state: ResMut<PopupState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>,
    total_completion: Res<TotalCompletion>,
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
        &total_completion,
        &mut pressed_button,
        &mut hint_state,
        &mut popup_state,
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
    mut pressed_button: ResMut<PressedButton>,
    mut hint_state: ResMut<HintState>,
    mut popup_state: ResMut<PopupState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>,
    total_completion: Res<TotalCompletion>,
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
            &total_completion,
            &mut pressed_button,
            &mut hint_state,
            &mut popup_state,
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
