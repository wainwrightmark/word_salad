use bevy::prelude::*;
use itertools::Either;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use std::time::Duration;
use strum::EnumIs;
use ws_core::layout::entities::{
    CongratsButton, CongratsLayoutEntity, LayoutTopBar, LayoutWordTile,
};

use crate::completion::TotalCompletion;
use crate::menu_layout::main_menu_back_button::MainMenuBackButton;
use crate::menu_layout::word_salad_menu_layout::WordSaladMenuLayoutEntity;
use crate::prelude::level_group_layout::LevelGroupLayoutEntity;
use crate::prelude::levels_menu_layout::LevelsMenuLayoutEntity;
use crate::prelude::main_menu_layout::MainMenuLayoutEntity;
use crate::{input, prelude::*, startup};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PressedButton::default());
        app.add_systems(Update, track_held_button);
        app.register_transition::<TransformScaleLens>();

        app.add_event::<ButtonActivated>();

        app.add_systems(
            Update,
            handle_button_activations
                .after(input::handle_mouse_input)
                .after(input::handle_touch_input)
                .after(track_held_button),
        );
    }
}

fn track_held_button(
    mut pressed_button: ResMut<PressedButton>,
    time: Res<Time>,
    mut event_writer: EventWriter<ButtonActivated>,
) {
    if pressed_button.is_none() {
        return;
    }

    let PressedButton::Pressed {
        interaction,
        duration,
    } = pressed_button.as_ref()
    else {
        return;
    };
    startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let interaction = *interaction;
    let duration = *duration + time.delta();

    //info!("{duration:?}");

    let ButtonPressType::OnHold(hold_duration) = interaction.button_press_type() else {
        *pressed_button.as_mut() = PressedButton::Pressed {
            interaction,
            duration,
        };
        return;
    };

    if duration >= hold_duration {
        *pressed_button = PressedButton::PressedAfterActivated { interaction };
        event_writer.send(ButtonActivated(interaction));
    } else {
        *pressed_button.as_mut() = PressedButton::Pressed {
            interaction,
            duration,
        };
    }
}

fn handle_button_activations(
    mut events: EventReader<ButtonActivated>,
    mut current_level: ResMut<CurrentLevel>,
    mut menu_state: ResMut<MenuState>,
    mut chosen_state: ResMut<ChosenState>,
    mut found_words: ResMut<FoundWordsState>,
    mut hint_state: ResMut<HintState>,
    mut popup_state: ResMut<PopupState>,
    video_state: Res<VideoResource>,
    video_events: AsyncEventWriter<VideoEvent>,
    mut total_completion: ResMut<TotalCompletion>,
    daily_challenges: Res<DailyChallenges>,
    mut level_time: ResMut<LevelTime>,
    mut selfie_mode_history: ResMut<SelfieModeHistory>,
    mut ew: EventWriter<AnimateSolutionsEvent>,
) {
    for ev in events.read() {
        ev.0.on_activated(
            &mut current_level,
            &mut menu_state,
            &mut chosen_state,
            &mut found_words,
            &mut hint_state,
            &mut popup_state,
            &mut total_completion,
            video_state.as_ref(),
            &video_events,
            daily_challenges.as_ref(),
            &mut level_time,
            &mut selfie_mode_history,
            &mut ew,
        )
    }
}

#[derive(Debug, PartialEq, Event)]
pub struct ButtonActivated(pub ButtonInteraction);

#[derive(Debug, Clone, Copy, PartialEq, Resource, Default, EnumIs)]
pub enum PressedButton {
    #[default]
    None,
    Pressed {
        interaction: ButtonInteraction,
        duration: Duration,
    },
    PressedAfterActivated {
        interaction: ButtonInteraction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum ButtonPressType {
    OnStart,
    OnHold(Duration),
    OnEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, EnumIs)]
pub enum PopupInteraction {
    ClickGreyedOut,
    ClickClose,
    HintsBuyMore,
    SelfieInformation,
    SelfieDontShowAgain,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, EnumIs, Default)]
pub enum ButtonInteraction {
    #[default]
    None,
    MainMenu(MainMenuLayoutEntity),
    LevelsMenu(LevelsMenuLayoutEntity),
    LevelGroupMenu(LevelGroupLayoutEntity),
    WordSaladMenu(WordSaladMenuLayoutEntity),
    WordButton(LayoutWordTile),
    TopMenuItem(LayoutTopBar),
    Congrats(CongratsButton),
    Popup(PopupInteraction),
    MenuBackButton,
    NonLevelInteractionButton,
    TimerButton,
}

impl ButtonInteraction {
    pub fn button_press_type(&self) -> ButtonPressType {
        if self.is_word_button() {
            ButtonPressType::OnHold(Duration::from_secs_f32(WORD_BUTTON_HOLD_SECONDS))
        } else if self.is_congrats() || *self == Self::Popup(PopupInteraction::ClickGreyedOut) {
            ButtonPressType::OnStart
        } else {
            ButtonPressType::OnEnd
        }
    }
}

impl From<MainMenuLayoutEntity> for ButtonInteraction {
    fn from(val: MainMenuLayoutEntity) -> Self {
        ButtonInteraction::MainMenu(val)
    }
}
impl From<LevelsMenuLayoutEntity> for ButtonInteraction {
    fn from(val: LevelsMenuLayoutEntity) -> Self {
        ButtonInteraction::LevelsMenu(val)
    }
}
impl From<WordSaladMenuLayoutEntity> for ButtonInteraction {
    fn from(val: WordSaladMenuLayoutEntity) -> Self {
        ButtonInteraction::WordSaladMenu(val)
    }
}
impl From<LevelGroupLayoutEntity> for ButtonInteraction {
    fn from(val: LevelGroupLayoutEntity) -> Self {
        ButtonInteraction::LevelGroupMenu(val)
    }
}
impl From<LayoutWordTile> for ButtonInteraction {
    fn from(val: LayoutWordTile) -> Self {
        ButtonInteraction::WordButton(val)
    }
}
impl From<LayoutTopBar> for ButtonInteraction {
    fn from(val: LayoutTopBar) -> Self {
        ButtonInteraction::TopMenuItem(val)
    }
}

impl From<CongratsButton> for ButtonInteraction {
    fn from(val: CongratsButton) -> Self {
        ButtonInteraction::Congrats(val)
    }
}

impl From<CongratsLayoutEntity> for ButtonInteraction {
    fn from(value: CongratsLayoutEntity) -> Self {
        match value {
            CongratsLayoutEntity::Statistic(_) => ButtonInteraction::None,
            CongratsLayoutEntity::Button(b) => b.into(),
        }
    }
}

impl From<MainMenuBackButton> for ButtonInteraction {
    fn from(_: MainMenuBackButton) -> Self {
        ButtonInteraction::MenuBackButton
    }
}

impl ButtonInteraction {
    /// Called when the button action is activated
    fn on_activated(
        &self,
        current_level: &mut ResMut<CurrentLevel>,
        menu_state: &mut ResMut<MenuState>,
        chosen_state: &mut ResMut<ChosenState>,
        found_words: &mut ResMut<FoundWordsState>,
        hint_state: &mut ResMut<HintState>,
        popup_state: &mut ResMut<PopupState>,

        total_completion: &mut ResMut<TotalCompletion>,
        video_state: &VideoResource,
        video_events: &AsyncEventWriter<VideoEvent>,
        daily_challenges: &DailyChallenges,
        level_time: &mut ResMut<LevelTime>,
        selfie_mode_history: &mut ResMut<SelfieModeHistory>,

        ew: &mut EventWriter<AnimateSolutionsEvent>,
    ) {
        match self {
            ButtonInteraction::None => {}

            ButtonInteraction::MenuBackButton => {
                menu_state.go_back();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::ResetPuzzle) => {
                current_level.set_changed();
                menu_state.close();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Puzzles) => {
                *menu_state.as_mut() = MenuState::ChooseLevelsPage;
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Store) => {
                //todo do something
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::SelfieMode) => {
                video_state.toggle_video_streaming(video_events.clone());
                menu_state.close();
            }

            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Tutorial) => {
                *current_level.as_mut() = CurrentLevel::NonLevel(NonLevel::BeforeTutorial);
                menu_state.close();
            }
            #[cfg(target_arch = "wasm32")]
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::PlaySteks) => {
                crate::wasm::open_link("https://steks.net");
            }

            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::WordSalad) => {
                *menu_state.as_mut() = MenuState::WordSaladLevels;
            }

            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::TodayPuzzle) => {
                if let Some(index) = DailyChallenges::get_today_index() {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge { index });
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::YesterdayPuzzle) => {
                if let Some(index) = DailyChallenges::get_today_index() {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge {
                        index: index.saturating_sub(1),
                    });
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::EreYesterdayPuzzle) => {
                if let Some(index) = DailyChallenges::get_today_index() {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge {
                        index: index.saturating_sub(2),
                    });
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::NextPuzzle) => {
                if let Some(index) = DailyChallenges::get_today_index()
                    .and_then(|x| x.checked_sub(3))
                    .and_then(|x| total_completion.get_next_incomplete_daily_challenge(x))
                {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge { index });
                } else {
                    current_level.set_if_neq(CurrentLevel::NonLevel(NonLevel::DailyChallengeReset));
                }
                menu_state.close();
            }

            ButtonInteraction::TimerButton => {
                if level_time.is_paused() {
                    level_time.as_mut().resume_timer();
                } else if level_time.is_running() {
                    level_time.as_mut().pause_timer();
                }
            }

            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::AdditionalLevel(group)) => {
                *menu_state.as_mut() = MenuState::LevelGroupPage(*group);
            }
            ButtonInteraction::LevelGroupMenu(entity) => match entity {
                LevelGroupLayoutEntity { index } => {
                    if let MenuState::LevelGroupPage(level_group) = menu_state.as_ref() {
                        let sequence = level_group.get_level_sequence(*index);

                        if let Some(index) = total_completion.get_next_level_index(sequence) {
                            info!("Changing level to {sequence} {index}");
                            *current_level.as_mut() = CurrentLevel::Fixed {
                                level_index: index,
                                sequence,
                            };
                        } else {
                            *current_level.as_mut() =
                                CurrentLevel::NonLevel(NonLevel::LevelSequenceReset(sequence));
                        }

                        menu_state.close();
                    }
                }
            },
            ButtonInteraction::WordButton(word) => {
                if hint_state.hints_remaining == 0 {
                    popup_state.0 = Some(PopupType::BuyMoreHints);
                } else if let Either::Left(level) = current_level.level(daily_challenges) {
                    found_words.try_hint_word(hint_state, level, word.0, chosen_state, ew);
                }
            }
            ButtonInteraction::TopMenuItem(LayoutTopBar::HintCounter) => {
                popup_state.0 = Some(PopupType::BuyMoreHints);
            }
            ButtonInteraction::TopMenuItem(LayoutTopBar::MenuBurgerButton) => menu_state.toggle(),
            ButtonInteraction::NonLevelInteractionButton => {
                if let Some(non_level) = current_level.level(daily_challenges).right() {
                    match non_level {
                        NonLevel::BeforeTutorial => {
                            *current_level.as_mut() = CurrentLevel::Tutorial { index: 0 };
                        }
                        NonLevel::AfterCustomLevel => {
                            if let Some(l) = CUSTOM_LEVEL.get() {
                                *current_level.as_mut() = CurrentLevel::Custom {
                                    name: l.name.to_string(),
                                };
                            }
                        }
                        NonLevel::DailyChallengeReset => {
                            total_completion.reset_daily_challenge_completion();
                            if let Some(index) =
                                total_completion.get_next_incomplete_daily_challenge_from_today()
                            {
                                *current_level.as_mut() = CurrentLevel::DailyChallenge { index };
                            }
                        }
                        NonLevel::LevelSequenceReset(ls) => {
                            total_completion.restart_level_sequence_completion(ls);
                            *current_level.as_mut() = CurrentLevel::Fixed {
                                level_index: 0,
                                sequence: ls,
                            };
                        }
                        NonLevel::DailyChallengeFinished => {
                            let new_current_level =
                                match total_completion.get_next_level_sequence(None) {
                                    Some((sequence, level_index)) => CurrentLevel::Fixed {
                                        level_index,
                                        sequence,
                                    },
                                    None => CurrentLevel::NonLevel(NonLevel::DailyChallengeReset),
                                };

                            *current_level.as_mut() = new_current_level;
                        }
                        NonLevel::LevelSequenceFinished(seq) => {
                            let new_current_level = match total_completion
                                .get_next_level_sequence(Some(seq))
                            {
                                Some((sequence, level_index)) => CurrentLevel::Fixed {
                                    level_index,
                                    sequence,
                                },
                                None => CurrentLevel::NonLevel(NonLevel::LevelSequenceReset(seq)),
                            };

                            *current_level.as_mut() = new_current_level;
                        }
                    }
                }
            }
            ButtonInteraction::TopMenuItem(LayoutTopBar::WordSaladLogo) => {
                if let Some(index) = DailyChallenges::get_today_index() {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge { index });
                }
                menu_state.close();
            }
            ButtonInteraction::Congrats(CongratsButton::Next) => {
                let next_level = current_level.get_next_level(total_completion);
                *current_level.as_mut() = next_level;
            }

            ButtonInteraction::Congrats(CongratsButton::MoreLevels) => {
                *menu_state.as_mut() = MenuState::ChooseLevelsPage;
            }

            #[cfg(target_arch = "wasm32")]
            ButtonInteraction::Congrats(CongratsButton::Share) => {
                if let Some(share_text) = try_generate_share_text(
                    current_level,
                    level_time.as_ref(),
                    found_words.as_ref(),
                    daily_challenges,
                ) {
                    crate::wasm::share(share_text);
                }
            }
            ButtonInteraction::Popup(
                PopupInteraction::ClickClose | PopupInteraction::ClickGreyedOut,
            ) => {
                popup_state.0 = None;
            }

            ButtonInteraction::Popup(PopupInteraction::HintsBuyMore) => {
                hint_state.hints_remaining += 3; //TODO actually make them buy them!
                hint_state.total_bought_hints += 3;
                popup_state.0 = None;
            }

            ButtonInteraction::Popup(PopupInteraction::SelfieInformation) => {
                #[cfg(target_arch = "wasm32")]
                {
                    let url = match Platform::CURRENT{
                        Platform::IOs => "https://support.apple.com/en-gb/HT207935#:~:text=Go%20to%20Settings%20%3E%20Control%20Centre,iPhone%2C%20or%20on%20your%20iPad.&text=%2C%20then%20wait%20for%20the%203%2Dsecond%20countdown",
                        Platform::Android => "https://support.google.com/android/answer/9075928?hl=en-GB",
                        Platform::Web => "https://support.apple.com/en-gb/HT207935#:~:text=Go%20to%20Settings%20%3E%20Control%20Centre,iPhone%2C%20or%20on%20your%20iPad.&text=%2C%20then%20wait%20for%20the%203%2Dsecond%20countdown", // todo look at device type
                        Platform::Other => "https://support.apple.com/en-gb/HT207935#:~:text=Go%20to%20Settings%20%3E%20Control%20Centre,iPhone%2C%20or%20on%20your%20iPad.&text=%2C%20then%20wait%20for%20the%203%2Dsecond%20countdown", //todo better links
                    };

                    crate::wasm::open_link(url);
                }
            }

            ButtonInteraction::Popup(PopupInteraction::SelfieDontShowAgain) => {
                selfie_mode_history.do_not_show_selfie_mode_tutorial = true;
                popup_state.0 = None;
            }
        }
    }
}

#[allow(dead_code)]

fn try_generate_share_text(
    current_level: &CurrentLevel,
    time: &LevelTime,
    found_words_state: &FoundWordsState,
    daily_challenges: &DailyChallenges,
) -> Option<String> {
    let level = current_level.level(daily_challenges).left()?;
    let first_lines = match level.numbering {
        Some(Numbering::WordSaladNumber(num)) => format!("Word Salad #{num}\n{}", level.name),
        Some(Numbering::SequenceNumber(..)) => level.full_name().to_string(),
        None => level.full_name().to_string(),
    };
    let total_secs = time.total_elapsed().as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    let hints = found_words_state.hints_used;
    let second_line = format!("⌛{minutes}m {seconds}s, ❓{hints}");

    let url = match current_level {
        CurrentLevel::DailyChallenge { index } => {
            format!("https://wordsalad.online/daily/{}", index + 1)
        }
        _ => {
            //todo data for sequence levels?
            "https://wordsalad.online/".to_string()
        }
    };

    Some(format!("{first_lines}\n{second_line}\n{url}"))
}
