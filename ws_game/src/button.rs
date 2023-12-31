use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use itertools::Either;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
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
use crate::{prelude::*, rounding};

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PressedButton::default());
        app.add_systems(Update, track_pressed_button);
        app.register_transition::<TransformScaleLens>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Resource, Default)]
pub struct PressedButton {
    pub interaction: Option<ButtonInteraction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum ButtonPressType {
    OnStart,
    OnEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, EnumIs)]
pub enum ButtonInteraction {
    MainMenu(MainMenuLayoutEntity),
    LevelsMenu(LevelsMenuLayoutEntity),
    LevelGroupMenu(LevelGroupLayoutEntity),
    WordSaladMenu(WordSaladMenuLayoutEntity),
    WordButton(LayoutWordTile),
    TopMenuItem(LayoutTopBar),
    Congrats(CongratsButton),
    BuyMoreHints,
    ClosePopups,
    MenuBackButton,
    NonLevelInteractionButton,
    TimerButton,
    None,
}

impl ButtonInteraction {
    pub fn button_press_type(&self) -> ButtonPressType {
        if self.is_congrats() {
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

fn track_pressed_button(
    mut commands: Commands,
    pressed_button: Res<PressedButton>,
    mut prev: Local<PressedButton>,
    query: Query<(Entity, &ButtonInteraction)>,
) {
    if !pressed_button.is_changed() {
        return;
    }
    let previous = *prev;
    *prev = *pressed_button;

    if let Some(prev_interaction) = previous.interaction {
        if Some(prev_interaction) == pressed_button.interaction {
            return;
        }

        for (entity, i) in query.iter() {
            if i == &prev_interaction {
                let mut ec = commands.entity(entity);
                i.on_interact(&mut ec, InteractionType::EndPress);
            }
        }
    }

    if let Some(interaction) = pressed_button.interaction {
        for (entity, i) in query.iter() {
            if i == &interaction {
                let mut ec = commands.entity(entity);
                i.on_interact(&mut ec, InteractionType::Press);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum InteractionType {
    Press,
    EndPress,
}

impl ButtonInteraction {
    pub fn on_interact(&self, commands: &mut EntityCommands, interaction_type: InteractionType) {
        let new_rounding = match (self, interaction_type) {
            (ButtonInteraction::WordButton(_), InteractionType::Press) => {
                rounding::WORD_BUTTON_PRESSED
            }
            (ButtonInteraction::WordButton(_), InteractionType::EndPress) => {
                rounding::WORD_BUTTON_NORMAL
            }

            (_, InteractionType::Press) => rounding::OTHER_BUTTON_PRESSED,
            (_, InteractionType::EndPress) => rounding::OTHER_BUTTON_NORMAL,
        };

        commands.insert(
            TransitionBuilder::<RoundingLens>::default()
                .then_tween(new_rounding, 1.0.into())
                .build(),
        );
    }

    pub fn on_pressed(
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
            #[cfg(target_arch= "wasm32")]

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
            ButtonInteraction::WordSaladMenu(WordSaladMenuLayoutEntity::NextPuzzle) => {
                if let Some(index) = DailyChallenges::get_today_index()
                    .and_then(|x| x.checked_sub(2))
                    .and_then(|x| total_completion.get_next_incomplete_daily_challenge(x))
                {
                    current_level.set_if_neq(CurrentLevel::DailyChallenge { index });
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
                                CurrentLevel::NonLevel(NonLevel::NoMoreLevelSequence(sequence));
                        }

                        menu_state.close();
                    }
                }
            },
            ButtonInteraction::WordButton(word) => {
                if hint_state.hints_remaining == 0 {
                    *popup_state.as_mut() = PopupState::BuyMoreHints;
                } else {
                    if let Either::Left(level) = current_level.level(daily_challenges) {
                        found_words.try_hint_word(hint_state, level, word.0, chosen_state);
                    }
                }
            }
            ButtonInteraction::TopMenuItem(LayoutTopBar::HintCounter) => {
                *popup_state.as_mut() = PopupState::BuyMoreHints;
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
                        NonLevel::NoMoreDailyChallenge => {
                            total_completion.reset_daily_challenge_completion();
                            if let Some(index) =
                                total_completion.get_next_incomplete_daily_challenge_from_today()
                            {
                                *current_level.as_mut() = CurrentLevel::DailyChallenge { index };
                            }
                        }
                        NonLevel::NoMoreLevelSequence(ls) => {
                            total_completion.restart_level_sequence_completion(ls);
                            *current_level.as_mut() = CurrentLevel::Fixed {
                                level_index: 0,
                                sequence: ls,
                            };
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

                if let Either::Left(level) = current_level.level(daily_challenges){
                    let share_text = generate_share_text(level, level_time.as_ref(), found_words.as_ref());
                    crate::wasm::share(share_text);
                }


            }

            ButtonInteraction::BuyMoreHints => {
                hint_state.hints_remaining += 3; //TODO actually make them buy them!
                hint_state.total_bought_hints += 3;
                *popup_state.as_mut() = PopupState::None;
            }
            ButtonInteraction::ClosePopups => {
                *popup_state.as_mut() = PopupState::None;
            }
        }
    }
}

#[allow(dead_code)]

fn generate_share_text(level: &DesignedLevel, time: &LevelTime, found_words_state: &FoundWordsState)-> String{

    let first_lines = match level.numbering{
        Some(Numbering::WordSaladNumber(num)) => format!("Word Salad #{num}\n{}", level.name),
        Some(Numbering::SequenceNumber(..)) => level.full_name().to_string(),
        None => level.full_name().to_string(),
    };
    let total_secs = time.total_elapsed().as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    let hints = found_words_state.hints_used;
    let second_line = format!("⌛{minutes}m {seconds}s, ❓{hints}");

    format!("{first_lines}\n{second_line}\nhttps://wordsalad.online/")
}