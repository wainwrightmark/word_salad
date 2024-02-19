use bevy::prelude::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use std::time::Duration;
use strum::EnumIs;
use ws_core::layout::entities::recording_button::ToggleRecordingButton;
use ws_core::layout::entities::{
    CongratsButton, CongratsLayoutEntity, LayoutWordTile, WordSaladLogo,
};
use ws_levels::level_group::LevelGroup;

use crate::menu_layout::main_menu_back_button::MainMenuBackButton;
use crate::menu_layout::word_salad_menu_layout::WordSaladMenuLayoutEntity;
use crate::prelude::level_group_layout::LevelGroupLayoutEntity;
use crate::prelude::levels_menu_layout::LevelsMenuLayoutEntity;
use crate::prelude::main_menu_layout::MainMenuLayoutEntity;
use crate::purchases::{PurchaseEvent, Purchases};
use crate::{achievements, asynchronous, completion::*};
use crate::{input, prelude::*, startup};

use self::hints_menu_layout::HintsLayoutEntity;
use self::settings_menu_layout::SettingsLayoutEntity;
use self::store_menu_layout::StoreLayoutStructure;

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
                .run_if(|ev: EventReader<ButtonActivated>| !ev.is_empty())
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
        start_elapsed,
        ..
    } = pressed_button.as_ref()
    else {
        return;
    };
    startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let interaction = *interaction;
    let held_duration = time.elapsed().saturating_sub(*start_elapsed);

    //info!("{duration:?}");

    let ButtonPressType::OnHold(hold_duration) = interaction.button_press_type() else {
        return;
    };

    if held_duration >= hold_duration {
        *pressed_button = PressedButton::PressedAfterActivated { interaction };
        event_writer.send(ButtonActivated(interaction));
    }
}

fn handle_button_activations(
    mut events: EventReader<ButtonActivated>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    mut menu_state: ResMut<MenuState>,
    mut popup_state: ResMut<PopupState>,
    mut sequence_completion: ResMut<SequenceCompletion>,
    mut daily_challenge_completion: ResMut<DailyChallengeCompletion>,
    purchases: Res<Purchases>,
    mut video_resource: ResMut<VideoResource>,

    daily_challenges: Res<DailyChallenges>,
    mut level_time: ResMut<LevelTime>,

    mut event_writers: (
        EventWriter<ChangeLevelEvent>,
        EventWriter<AdRequestEvent>,
        EventWriter<PurchaseEvent>,
        EventWriter<HintEvent>,
        AsyncEventWriter<VideoEvent>,
        AsyncEventWriter<DailyChallengeDataLoadedEvent>,
    ),
) {
    for ev in events.read() {
        ev.0.on_activated(
            &current_level,
            &found_words,
            &mut menu_state,
            &mut popup_state,
            &mut sequence_completion,
            &mut daily_challenge_completion,
            &mut video_resource,
            daily_challenges.as_ref(),
            &mut level_time,
            &purchases,
            &mut event_writers.0,
            &mut event_writers.1,
            &mut event_writers.2,
            &mut event_writers.3,
            &event_writers.4,
            &mut event_writers.5,
        )
    }
}

#[derive(Debug, PartialEq, Event)]
pub struct ButtonActivated(pub ButtonInteraction);

#[derive(Debug, Clone, Copy, PartialEq, Resource, Default, EnumIs, MavericContext)]
pub enum PressedButton {
    #[default]
    None,
    NoInteractionPressed {
        start_state: StartPressState,
    },
    Pressed {
        interaction: ButtonInteraction,
        start_elapsed: Duration,
        start_state: StartPressState,
    },
    PressedAfterActivated {
        interaction: ButtonInteraction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum StartPressState {
    Gameplay,
    Congrats,
    Menu,
    Popup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum ButtonPressType {
    // OnStart,
    OnHold(Duration),
    OnEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, EnumIs)]
pub enum PopupInteraction {
    ClickSufferAlone,
    ClickClose,
    ClickWatchAd,
    ClickHintsStore,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, EnumIs, Default)]
pub enum ButtonInteraction {
    #[default]
    None,
    MainMenu(MainMenuLayoutEntity),
    LevelsMenu(LevelsMenuLayoutEntity),
    LevelGroupMenu(LevelGroupLayoutEntity),
    WordSaladMenu(WordSaladMenuLayoutEntity),
    MainStoreMenu(StoreLayoutStructure),
    BuyLevelGroup(LevelGroup),
    HintsMenu(HintsLayoutEntity),
    SettingsMenu(SettingsLayoutEntity),
    WordButton(LayoutWordTile),

    WordSaladLogo,
    ToggleRecordingButton,
    Congrats(CongratsButton),
    Popup(PopupInteraction),
    MenuBackButton,
    CloseMenu,
    NonLevelInteractionButton,
    TimerButton,
}

impl ButtonInteraction {
    pub fn button_press_type(&self) -> ButtonPressType {
        if self.is_word_button() {
            ButtonPressType::OnHold(Duration::from_secs_f32(WORD_BUTTON_HOLD_SECONDS))
        } else {
            ButtonPressType::OnEnd
        }
    }
}

impl From<level_group_store_layout::LevelGroupStoreLayoutStructure> for ButtonInteraction {
    fn from(value: level_group_store_layout::LevelGroupStoreLayoutStructure) -> Self {
        Self::BuyLevelGroup(value.0)
    }
}

impl From<HintsLayoutEntity> for ButtonInteraction {
    fn from(value: HintsLayoutEntity) -> Self {
        ButtonInteraction::HintsMenu(value)
    }
}

impl From<StoreLayoutStructure> for ButtonInteraction {
    fn from(value: StoreLayoutStructure) -> Self {
        ButtonInteraction::MainStoreMenu(value)
    }
}

impl From<SettingsLayoutEntity> for ButtonInteraction {
    fn from(value: SettingsLayoutEntity) -> Self {
        ButtonInteraction::SettingsMenu(value)
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
impl From<WordSaladLogo> for ButtonInteraction {
    fn from(_: WordSaladLogo) -> Self {
        ButtonInteraction::WordSaladLogo
    }
}
impl From<ToggleRecordingButton> for ButtonInteraction {
    fn from(_: ToggleRecordingButton) -> Self {
        ButtonInteraction::ToggleRecordingButton
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
            CongratsLayoutEntity::Statistic(_) | CongratsLayoutEntity::Time => {
                ButtonInteraction::None
            }
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
        current_level: &CurrentLevel,
        _found_words: &FoundWordsState, //needed for share
        menu_state: &mut ResMut<MenuState>,
        popup_state: &mut ResMut<PopupState>,

        sequence_completion: &mut ResMut<SequenceCompletion>,
        daily_challenge_completion: &mut ResMut<DailyChallengeCompletion>,
        video_resource: &mut ResMut<VideoResource>,

        daily_challenges: &DailyChallenges,
        level_time: &mut ResMut<LevelTime>,
        purchases: &Purchases,

        change_level_events: &mut EventWriter<ChangeLevelEvent>,
        ad_request_events: &mut EventWriter<AdRequestEvent>,
        purchase_events: &mut EventWriter<PurchaseEvent>,
        hint_events: &mut EventWriter<HintEvent>,
        video_events: &AsyncEventWriter<VideoEvent>,
        daily_challenge_events: &AsyncEventWriter<DailyChallengeDataLoadedEvent>,
    ) {
        match self {
            ButtonInteraction::None => {}

            ButtonInteraction::CloseMenu => {
                menu_state.close();
            }

            ButtonInteraction::MenuBackButton => {
                menu_state.go_back();
            }

            ButtonInteraction::MainMenu(MainMenuLayoutEntity::ResetPuzzle) => {
                change_level_events.send(ChangeLevelEvent::Reset);
                menu_state.close();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Puzzles) => {
                *menu_state.as_mut() = MenuState::ChooseLevelsPage;
            }
            #[cfg(target_arch = "wasm32")]
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Store) => {
                *menu_state.as_mut() = MenuState::MainStorePage;
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::SelfieMode) => {
                video_resource.toggle_selfie_mode(video_events.clone());
                menu_state.close();
            }

            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Tutorial) => {
                change_level_events.send(CurrentLevel::NonLevel(NonLevel::BeforeTutorial).into());
                menu_state.close();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Settings) => {
                *menu_state.as_mut() = MenuState::SettingsPage;
            }
            ButtonInteraction::SettingsMenu(SettingsLayoutEntity::SeeAchievements) => {
                achievements::show_achievements();
            }
            ButtonInteraction::SettingsMenu(SettingsLayoutEntity::AdsConsent) => {
                ad_request_events.send(AdRequestEvent::RequestConsent);
            }

            #[cfg(target_arch = "wasm32")]
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::PlaySteks) => {
                crate::wasm::open_link("https://steks.net");
            }

            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::WordSalad) => {
                *menu_state.as_mut() = MenuState::WordSaladLevels;
            }

            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::TodayPuzzle) => {
                let index = DailyChallenges::get_today_index();
                {
                    change_level_events.send(CurrentLevel::DailyChallenge { index }.into());
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::YesterdayPuzzle) => {
                let index = DailyChallenges::get_today_index();
                {
                    change_level_events.send(
                        CurrentLevel::DailyChallenge {
                            index: index.saturating_sub(1),
                        }
                        .into(),
                    );
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::EreYesterdayPuzzle) => {
                let index = DailyChallenges::get_today_index();
                {
                    change_level_events.send(
                        CurrentLevel::DailyChallenge {
                            index: index.saturating_sub(2),
                        }
                        .into(),
                    );
                }
                menu_state.close();
            }
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::NextPuzzle) => {
                if let Some(level) =
                    DailyChallenges::get_today_index()
                        .checked_sub(3)
                        .and_then(|x| {
                            daily_challenge_completion
                                .get_next_incomplete_daily_challenge(x, daily_challenges)
                                .actual_level()
                        })
                {
                    change_level_events.send(level.into());
                } else {
                    change_level_events
                        .send(CurrentLevel::NonLevel(NonLevel::DailyChallengeReset).into());
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

                        let level = sequence_completion
                            .get_next_level_index(sequence, purchases)
                            .to_level(sequence);

                        change_level_events.send(level.into());

                        menu_state.close();
                    }
                }
            },
            ButtonInteraction::WordButton(word) => {
                hint_events.send(HintEvent { word_index: word.0 });
            }

            ButtonInteraction::ToggleRecordingButton => {
                if video_resource.is_selfie_mode {
                    if video_resource.is_recording {
                        asynchronous::spawn_and_run(crate::video::stop_screen_record(
                            video_events.clone(),
                        ));
                    } else {
                        asynchronous::spawn_and_run(crate::video::start_screen_record(
                            video_events.clone(),
                        ));
                    }
                }
            }

            ButtonInteraction::NonLevelInteractionButton => {
                if let Some(non_level) = current_level.level(daily_challenges).right() {
                    match non_level {
                        NonLevel::BeforeTutorial => {
                            change_level_events.send(CurrentLevel::Tutorial { index: 0 }.into());
                        }
                        NonLevel::AfterCustomLevel => {
                            if let Some(l) = CUSTOM_LEVEL.get() {
                                change_level_events.send(
                                    CurrentLevel::Custom {
                                        name: l.name.to_string(),
                                    }
                                    .into(),
                                );
                            }
                        }
                        NonLevel::LevelSequenceMustPurchaseGroup(sequence) => {
                            purchase_events.send(PurchaseEvent::BuyLevelGroupBySequence(sequence));
                        }
                        NonLevel::DailyChallengeReset => {
                            daily_challenge_completion.reset_daily_challenge_completion();
                            match daily_challenge_completion
                                .get_next_incomplete_daily_challenge_from_today(daily_challenges)
                            {
                                NextDailyChallengeResult::Level(index) => {
                                    change_level_events
                                        .send(CurrentLevel::DailyChallenge { index }.into());
                                }
                                _ => {}
                            }
                        }
                        NonLevel::LevelSequenceReset(ls) => {
                            sequence_completion.restart_level_sequence_completion(ls);
                            change_level_events.send(
                                CurrentLevel::Fixed {
                                    level_index: 0,
                                    sequence: ls,
                                }
                                .into(),
                            );
                        }
                        NonLevel::DailyChallengeCountdown { todays_index } => {
                            change_level_events.send(
                                CurrentLevel::DailyChallenge {
                                    index: todays_index,
                                }
                                .into(),
                            );
                        }
                        NonLevel::DailyChallengeFinished => {
                            let new_current_level = match sequence_completion
                                .get_next_level_sequence(None, purchases)
                            {
                                Some((sequence, level_index)) => CurrentLevel::Fixed {
                                    level_index,
                                    sequence,
                                },
                                None => CurrentLevel::NonLevel(NonLevel::DailyChallengeReset),
                            };

                            change_level_events.send(new_current_level.into());
                        }
                        NonLevel::LevelSequenceAllFinished(seq) => {
                            let new_current_level = match sequence_completion
                                .get_next_level_sequence(Some(seq), purchases)
                            {
                                Some((sequence, level_index)) => CurrentLevel::Fixed {
                                    level_index,
                                    sequence,
                                },
                                None => CurrentLevel::NonLevel(NonLevel::LevelSequenceReset(seq)),
                            };

                            change_level_events.send(new_current_level.into());
                        }
                        NonLevel::DailyChallengeNotLoaded { goto_level } => {
                            asynchronous::spawn_and_run(load_levels_async(
                                daily_challenge_events.clone(),
                                true,
                            ));
                            change_level_events.send(
                                CurrentLevel::NonLevel(NonLevel::DailyChallengeLoading {
                                    goto_level,
                                })
                                .into(),
                            );
                        }
                        NonLevel::DailyChallengeLoading { .. } => {
                            //This button should not exist
                        }
                    }
                }
            }
            ButtonInteraction::BuyLevelGroup(level_group) => {
                if !purchases.groups_purchased.contains(level_group){
                    purchase_events.send(PurchaseEvent::BuyLevelGroup(*level_group));
                    menu_state.close();
                }

            }

            ButtonInteraction::MainStoreMenu(m) => match m {
                StoreLayoutStructure::RemoveAds => {
                    purchase_events.send(PurchaseEvent::BuyAvoidAds);
                    menu_state.close();
                }
                StoreLayoutStructure::BuyHints => {
                    *menu_state.as_mut() = MenuState::HintsStorePage;
                }
                StoreLayoutStructure::LevelGroups => {
                    *menu_state.as_mut() = MenuState::LevelGroupStorePage
                }
            },
            ButtonInteraction::HintsMenu(hints_layout_entity) => {
                let (method, hints) = hints_layout_entity.hint_data();

                match method {
                    hints_menu_layout::PurchaseMethod::WatchAd => {
                        ad_request_events
                            .send(AdRequestEvent::RequestReward { event: None, hints });
                    }
                    hints_menu_layout::PurchaseMethod::Money => {
                        purchase_events.send(PurchaseEvent::BuyHintsPack {
                            hint_event: None,
                            number_of_hints: hints,
                        });
                    }
                }

                menu_state.close();
            }

            ButtonInteraction::WordSaladLogo => menu_state.toggle(),
            ButtonInteraction::Congrats(CongratsButton::Next) => {
                let next_level = current_level.get_next_level(
                    daily_challenge_completion,
                    sequence_completion,
                    purchases,
                    daily_challenges,
                );
                change_level_events.send(next_level.into());
            }

            ButtonInteraction::Congrats(CongratsButton::MoreLevels) => {
                *menu_state.as_mut() = MenuState::ChooseLevelsPage;
            }
            ButtonInteraction::Congrats(CongratsButton::ResetPuzzle) => {
                change_level_events.send(ChangeLevelEvent::Reset);
            }

            #[cfg(target_arch = "wasm32")]
            ButtonInteraction::Congrats(CongratsButton::Share) => {
                if let Some(share_text) = try_generate_share_text(
                    current_level,
                    level_time.as_ref(),
                    _found_words,
                    daily_challenges,
                ) {
                    crate::wasm::share(share_text);
                }
            }
            ButtonInteraction::Popup(
                PopupInteraction::ClickClose | PopupInteraction::ClickSufferAlone,
            ) => {
                popup_state.0 = None;
            }

            ButtonInteraction::Popup(PopupInteraction::ClickWatchAd) => {
                if let Some(PopupType::BuyMoreHints(he)) = popup_state.0.take() {
                    ad_request_events.send(AdRequestEvent::RequestReward {
                        event: Some(he),
                        hints: 5,
                    });
                }
            }

            ButtonInteraction::Popup(PopupInteraction::ClickHintsStore) => {
                if let Some(PopupType::BuyMoreHints(..)) = popup_state.0.take() {
                    menu_state.set_if_neq(MenuState::HintsStorePage);
                }
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

    Some(format!("{url}\n{first_lines}\n{second_line}"))
}
