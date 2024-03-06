use crate::{
    achievements::UserSignedIn,
    menu_layout::{
        main_menu_back_button::MainMenuBackButton,
        word_salad_menu_layout::WordSaladMenuLayoutEntity,
    },
    prelude::{
        level_group_layout::LevelGroupLayoutEntity, levels_menu_layout::LevelsMenuLayoutEntity,
        main_menu_layout::MainMenuLayoutEntity, *,
    },
    startup,
};
use bevy::{prelude::*, window::PrimaryWindow};
use strum::EnumIs;
use ws_core::layout::entities::{level_info_entity::IsLevelComplete, recording_button::ToggleRecordingButton, *};

use self::{
    hints_menu_layout::HintsLayoutEntity, settings_menu_layout::SettingsLayoutEntity,
    store_menu_layout::StoreLayoutStructure,
};
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
        context: &T::Context<'_>,
    ) -> Option<Self> {
        size.try_pick::<T>(*position, context)
            .map(|x| InteractionEntity::Button(x.into()))
    }

    pub fn try_find(
        position: &Vec2,
        size: &Size,
        menu_state: &MenuState,
        popup_state: &PopupState,
        current_level: &CurrentLevel,
        daily_challenges: &DailyChallenges,
        video_resource: &VideoResource,
        is_level_complete: bool,
        is_game_paused: bool,
        grid_tolerance: Option<f32>,
        user_signed_in: &UserSignedIn,
        insets: Insets,
    ) -> Option<Self> {
        let selfie_mode = SelfieMode {
            is_selfie_mode: video_resource.is_selfie_mode,
        };
        //info!("Try find input");
        if let Some(popup_type) = popup_state.0 {
            match popup_type {
                PopupType::BuyMoreHints(_) => {
                    return match size.try_pick::<HintsPopupLayoutEntity>(*position, &()) {
                        Some(entity) => match entity {
                            HintsPopupLayoutEntity::Text => None,
                            HintsPopupLayoutEntity::SufferAloneButton => {
                                Some(InteractionEntity::Button(ButtonInteraction::Popup(
                                    PopupInteraction::ClickClose,
                                )))
                            }
                            HintsPopupLayoutEntity::PopupBox => None,
                            HintsPopupLayoutEntity::WatchAdButton => {
                                Some(InteractionEntity::Button(ButtonInteraction::Popup(
                                    PopupInteraction::ClickWatchAd,
                                )))
                            }
                            HintsPopupLayoutEntity::HintsStorePageButton => {
                                Some(InteractionEntity::Button(ButtonInteraction::Popup(
                                    PopupInteraction::ClickHintsStore,
                                )))
                            }
                        },
                        None => Some(InteractionEntity::Button(ButtonInteraction::Popup(
                            PopupInteraction::ClickSufferAlone,
                        ))),
                    }
                }
            }
        }

        let tbi = Self::try_get_button::<WordSaladLogo>(position, size, &((selfie_mode, insets), IsLevelComplete(is_level_complete)));
        if tbi.is_some() {
            return tbi;
        }

        if video_resource.show_recording_button()
            && size
                .try_pick::<ToggleRecordingButton>(*position, &(selfie_mode, insets))
                .is_some()
        {
            return Some(InteractionEntity::Button(
                ButtonInteraction::ToggleRecordingButton,
            ));
        }

        match menu_state {
            MenuState::Closed => match current_level.level(daily_challenges) {
                itertools::Either::Left(level) => {
                    if is_level_complete {
                        return Self::try_get_button::<CongratsLayoutEntity>(
                            position,
                            size,
                            &(selfie_mode, current_level.level_type()),
                        );
                    }

                    let Some(layout_entity) =
                        size.try_pick::<GameLayoutEntity>(*position, &(selfie_mode, insets))
                    else {
                        return None;
                    };

                    if is_game_paused {
                        return Some(InteractionEntity::Button(ButtonInteraction::TimerButton));
                    }

                    match layout_entity {
                        GameLayoutEntity::TopBar => {
                            if let Some(x) = Self::try_get_button::<WordSaladLogo>(
                                position,
                                size,
                                &((selfie_mode, insets), IsLevelComplete(is_level_complete)),
                            ) {
                                Some(x)
                            } else if let Some(x) = Self::try_get_button::<TimerLayoutEntity>(
                                position,
                                size,
                                &(selfie_mode, insets),
                            ) {
                                Some(x)
                            } else {
                                None
                            }
                        }

                        GameLayoutEntity::Grid => match grid_tolerance {
                            Some(tolerance) => size
                                .try_pick_with_tolerance::<LayoutGridTile>(
                                    *position,
                                    tolerance,
                                    &(selfie_mode, insets),
                                )
                                .map(|t| Self::Tile(t.0)),
                            None => size
                                .try_pick::<LayoutGridTile>(*position, &(selfie_mode, insets))
                                .map(|t| Self::Tile(t.0)),
                        },
                        GameLayoutEntity::WordList => Self::try_get_button::<LayoutWordTile>(
                            position,
                            size,
                            &(level.words.as_slice(), (selfie_mode, insets)),
                        ),
                        GameLayoutEntity::LevelInfo => None,
                    }
                }
                itertools::Either::Right(..) => {
                    let non_level_entity =
                        size.try_pick::<NonLevelLayoutEntity>(*position, &selfie_mode)?;

                    match non_level_entity {
                        NonLevelLayoutEntity::Text => None,
                        NonLevelLayoutEntity::InteractButton => Some(InteractionEntity::Button(
                            ButtonInteraction::NonLevelInteractionButton,
                        )),
                    }
                }
            },

            MenuState::SettingsPage => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<SettingsLayoutEntity>(
                        position,
                        size,
                        &(selfie_mode, *user_signed_in),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }

            MenuState::HintsStorePage => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<HintsLayoutEntity>(position, size, &(selfie_mode, ()))
                        .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }

            MenuState::LevelGroupStorePage => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<level_group_store_layout::LevelGroupStoreLayoutStructure>(position, size, &(selfie_mode, ()))
                        .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }

            MenuState::MainStorePage => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<StoreLayoutStructure>(
                        position,
                        size,
                        &(selfie_mode, ()),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }

            MenuState::ShowMainMenu => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<MainMenuLayoutEntity>(
                        position,
                        size,
                        &(selfie_mode, ()),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }
            MenuState::ChooseLevelsPage => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<LevelsMenuLayoutEntity>(
                        position,
                        size,
                        &(selfie_mode, ()),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }
            MenuState::WordSaladLevels => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<WordSaladMenuLayoutEntity>(
                        position,
                        size,
                        &(selfie_mode, ()),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }
            MenuState::LevelGroupPage(group) => {
                if let Some(back) = Self::try_get_button::<MainMenuBackButton>(position, size, &())
                {
                    return Some(back);
                }

                Some(
                    Self::try_get_button::<LevelGroupLayoutEntity>(
                        position,
                        size,
                        &(selfie_mode, *group),
                    )
                    .unwrap_or(InteractionEntity::Button(ButtonInteraction::CloseMenu)),
                )
            }
        }
    }
}

impl InputType {
    pub fn handle(
        &self,

        size: &Size,
        current_level: &CurrentLevel,
        menu_state: &MenuState,
        chosen_state: &mut ResMut<ChosenState>,
        input_state: &mut Local<GridInputState>,
        found_words: &FoundWordsState,
        pressed_button: &mut ResMut<PressedButton>,
        popup_state: &PopupState,
        video_resource: &VideoResource,
        daily_challenges: &DailyChallenges,
        event_writer: &mut EventWriter<ButtonActivated>,
        timer: &LevelTime,
        time: &Time,
        user_signed_in: &UserSignedIn,
        insets: &InsetsResource,
    ) {
        startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let is_level_complete = found_words.is_level_complete();

        let current_state: StartPressState = {
            if popup_state.0.is_some() {
                StartPressState::Popup
            } else if !menu_state.is_closed() {
                StartPressState::Menu
            } else if found_words.is_level_complete() {
                StartPressState::Congrats
            } else {
                StartPressState::Gameplay
            }
        };

        let button_interaction: Option<ButtonInteraction> = match self {
            InputType::Start(position) => {
                if let Some(interaction) = InteractionEntity::try_find(
                    position,
                    size,
                    menu_state,
                    popup_state,
                    current_level,
                    daily_challenges,
                    video_resource,
                    is_level_complete,
                    timer.is_paused(),
                    None,
                    user_signed_in,
                    insets.0,
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => Some(button),
                        InteractionEntity::Tile(tile) => {
                            if let Some(level) = current_level.level(daily_challenges).left() {
                                input_state.handle_input_start(
                                    chosen_state,
                                    tile,
                                    &level.grid,
                                    found_words,
                                );
                            }
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
                    popup_state,
                    current_level,
                    daily_challenges,
                    video_resource,
                    is_level_complete,
                    timer.is_paused(),
                    Some(MOVE_TOLERANCE),
                    user_signed_in,
                    insets.0,
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => Some(button),
                        InteractionEntity::Tile(tile) => {
                            if let Some(level) = current_level.level(daily_challenges).left() {
                                input_state.handle_input_move(
                                    chosen_state,
                                    tile,
                                    &level.grid,
                                    found_words,
                                );
                            };

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
                    popup_state,
                    current_level,
                    daily_challenges,
                    video_resource,
                    is_level_complete,
                    timer.is_paused(),
                    None,
                    user_signed_in,
                    insets.0,
                ) {
                    match interaction {
                        InteractionEntity::Button(button) => {
                            if button.button_press_type().is_on_end() {
                                match pressed_button.as_ref() {
                                    PressedButton::None
                                    | PressedButton::NoInteractionPressed { .. }
                                    | PressedButton::PressedAfterActivated { .. } => {}
                                    PressedButton::Pressed { start_state, .. } => {
                                        if start_state == &current_state {
                                            event_writer.send(ButtonActivated(button));
                                        }
                                    }
                                }
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

        match button_interaction {
            Some(new_interaction) => {
                let should_change = match pressed_button.as_ref() {
                    PressedButton::None => self.is_start(),
                    PressedButton::NoInteractionPressed { start_state } => {
                        current_state == *start_state
                    }
                    PressedButton::Pressed { interaction, .. } => *interaction != new_interaction,
                    PressedButton::PressedAfterActivated { .. } => {
                        false
                        // *interaction != new_interaction
                    }
                };

                if should_change {
                    *pressed_button.as_mut() = PressedButton::Pressed {
                        interaction: new_interaction,
                        start_elapsed: time.elapsed(),
                        start_state: current_state,
                    };

                    //info!("Changed button state to {:?}", *pressed_button)
                }
            }

            None => {
                let new_state = match self {
                    InputType::Start(_) => PressedButton::NoInteractionPressed {
                        start_state: current_state,
                    },
                    InputType::Move(_) => {
                        if let Some(start_state) = match pressed_button.as_ref() {
                            PressedButton::None | PressedButton::PressedAfterActivated { .. } => {
                                None
                            }
                            PressedButton::NoInteractionPressed { start_state }
                            | PressedButton::Pressed { start_state, .. } => Some(*start_state),
                        } {
                            if start_state == current_state {
                                PressedButton::NoInteractionPressed {
                                    start_state: current_state,
                                }
                            } else {
                                PressedButton::None
                            }
                        } else {
                            PressedButton::None
                        }
                    }
                    InputType::End(_) => PressedButton::None,
                };

                pressed_button.set_if_neq(new_state);
            }
        };
    }
}

pub fn handle_mouse_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,

    size: Res<Size>,
    current_level: Res<CurrentLevel>,
    menu_state: Res<MenuState>,
    mut chosen_state: ResMut<ChosenState>,
    mut input_state: Local<GridInputState>,
    found_words: Res<FoundWordsState>,
    mut pressed_button: ResMut<PressedButton>,
    popup_state: Res<PopupState>,
    video_resource: Res<VideoResource>,
    daily_challenges: Res<DailyChallenges>,
    timer: Res<LevelTime>,
    mut event_writer: EventWriter<ButtonActivated>,
    extras: (Res<Time>, Res<UserSignedIn>, Res<InsetsResource>),
) {
    let (time, user_signed_in, insets) = extras;

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
        &current_level,
        &menu_state,
        &mut chosen_state,
        &mut input_state,
        &found_words,
        &mut pressed_button,
        &popup_state,
        &video_resource,
        &daily_challenges,
        &mut event_writer,
        &timer,
        &time,
        &user_signed_in,
        &insets,
    );
}

pub fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    size: Res<Size>,

    current_level: Res<CurrentLevel>,
    menu_state: Res<MenuState>,
    mut chosen_state: ResMut<ChosenState>,
    mut input_state: Local<GridInputState>,
    found_words: Res<FoundWordsState>,
    mut pressed_button: ResMut<PressedButton>,
    popup_state: Res<PopupState>,
    video_resource: Res<VideoResource>,
    daily_challenges: Res<DailyChallenges>,
    timer: Res<LevelTime>,
    mut event_writer: EventWriter<ButtonActivated>,
    extras: (Res<Time>, Res<UserSignedIn>, Res<InsetsResource>),
) {
    let (time, user_signed_in, insets) = extras;

    for ev in touch_events.read() {
        let input_type: InputType = match ev.phase {
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
            &current_level,
            &menu_state,
            &mut chosen_state,
            &mut input_state,
            &found_words,
            &mut pressed_button,
            &popup_state,
            &video_resource,
            &daily_challenges,
            &mut event_writer,
            &timer,
            &time,
            &user_signed_in,
            &insets,
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
