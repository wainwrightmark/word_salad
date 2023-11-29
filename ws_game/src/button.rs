use bevy::prelude::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
use ws_core::layout::entities::{CongratsLayoutEntity, LayoutTopBarButton, LayoutWordTile};
use ws_levels::level_sequence::LevelSequence;

use crate::completion::TotalCompletion;
use crate::prelude::level_group_layout::LevelGroupLayoutEntity;
use crate::prelude::levels_menu_layout::LevelsMenuLayoutEntity;
use crate::prelude::main_menu_layout::MainMenuLayoutEntity;
use crate::prelude::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub enum ButtonInteraction {
    MainMenu(MainMenuLayoutEntity),
    LevelsMenu(LevelsMenuLayoutEntity),
    LevelGroupMenu(LevelGroupLayoutEntity),
    WordButton(LayoutWordTile),
    TopMenuItem(LayoutTopBarButton),
    Congrats(CongratsLayoutEntity),
    BuyMoreHints,
    ClosePopups,
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
impl From<LayoutTopBarButton> for ButtonInteraction {
    fn from(val: LayoutTopBarButton) -> Self {
        ButtonInteraction::TopMenuItem(val)
    }
}

impl From<CongratsLayoutEntity> for ButtonInteraction {
    fn from(val: CongratsLayoutEntity) -> Self {
        ButtonInteraction::Congrats(val)
    }
}

fn track_pressed_button(
    mut commands: Commands,
    pressed_button: Res<PressedButton>,
    mut prev: Local<PressedButton>,
    mut query: Query<(Entity, &ButtonInteraction)>,
) {
    if !pressed_button.is_changed() {
        return;
    }
    let previous = prev.clone();
    *prev = pressed_button.clone();

    if let Some(prev_interaction) = previous.interaction {
        if Some(prev_interaction) == pressed_button.interaction {
            return;
        }

        if let Some((entity, _)) = query.iter_mut().filter(|x| x.1 == &prev_interaction).next() {
            commands
                .entity(entity)
                .insert(Transition::<SmudParamLens<2>>::new(
                    TransitionStep::new_arc(0.1, Some(0.1.into()), NextStep::None),
                ));
        }
    }

    if let Some(interaction) = pressed_button.as_ref().interaction {
        if let Some((entity, _)) = query.iter_mut().filter(|x| x.1 == &interaction).next() {
            commands
                .entity(entity)
                .insert(Transition::<SmudParamLens<2>>::new(
                    TransitionStep::new_arc(0.15, Some(0.5.into()), NextStep::None),
                ));
        }
    }
}

impl ButtonInteraction {
    pub fn on_pressed(
        &self,
        current_level: &mut ResMut<CurrentLevel>,
        menu_state: &mut ResMut<MenuState>,
        chosen_state: &mut ResMut<ChosenState>,
        found_words: &mut ResMut<FoundWordsState>,
        mut hint_state: &mut ResMut<HintState>,
        popup_state: &mut ResMut<PopupState>,

        total_completion: &TotalCompletion,
        video_state: &VideoResource,
        video_events: &AsyncEventWriter<VideoEvent>,
    ) {
        match self {
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Resume) => {
                menu_state.close();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::ResetLevel) => {
                *found_words.as_mut() = FoundWordsState::new_from_level(current_level);
                *chosen_state.as_mut() = ChosenState::default();
                current_level.set_changed();
                menu_state.close();
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::ChooseLevel) => {
                *menu_state.as_mut() = MenuState::ChooseLevelsPage;
            }
            ButtonInteraction::MainMenu(MainMenuLayoutEntity::Video) => {
                video_state.toggle_video_streaming(video_events.clone());
            }
            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::Back) => {
                *menu_state.as_mut() = MenuState::ShowMainMenu;
            }
            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::DailyChallenge) => {
                current_level.to_level(
                    LevelSequence::DailyChallenge,
                    total_completion,
                    found_words,
                    chosen_state,
                );
                menu_state.close();
            }
            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::Tutorial) => {
                current_level.to_level(
                    LevelSequence::Tutorial,
                    total_completion,
                    found_words,
                    chosen_state,
                );
                menu_state.close();
            }
            ButtonInteraction::LevelsMenu(LevelsMenuLayoutEntity::AdditionalLevel(group)) => {
                *menu_state.as_mut() = MenuState::LevelGroupPage(*group);
            }
            ButtonInteraction::LevelGroupMenu(entity) => match entity {
                LevelGroupLayoutEntity::Level { index } => {
                    if let MenuState::LevelGroupPage(level_group) = menu_state.as_ref() {
                        let sequence = level_group.get_level_sequence(*index);
                        current_level.to_level(
                            sequence,
                            total_completion,
                            found_words,
                            chosen_state,
                        );
                        menu_state.close();
                    }
                }
                LevelGroupLayoutEntity::Back => {
                    *menu_state.as_mut() = MenuState::ChooseLevelsPage;
                }
            },
            ButtonInteraction::WordButton(word) => {
                if hint_state.hints_remaining <= 0 {
                    *popup_state.as_mut() = PopupState::BuyMoreHints;
                } else {
                    found_words.try_hint_word(&mut hint_state, &current_level, word.0);
                }
            }
            ButtonInteraction::TopMenuItem(LayoutTopBarButton::HintCounter) => {
                *popup_state.as_mut() = PopupState::BuyMoreHints;
            }
            ButtonInteraction::TopMenuItem(LayoutTopBarButton::MenuBurgerButton) => {
                menu_state.toggle()
            }
            ButtonInteraction::TopMenuItem(LayoutTopBarButton::TimeCounter) => {}
            ButtonInteraction::Congrats(CongratsLayoutEntity::NextButton) => {
                current_level.to_next_level(total_completion, found_words, chosen_state);
            }
            ButtonInteraction::Congrats(CongratsLayoutEntity::TimeCounter) => {}
            ButtonInteraction::Congrats(CongratsLayoutEntity::HintsUsed) => {}
            ButtonInteraction::Congrats(CongratsLayoutEntity::ShareButton) => {
                #[cfg(target_arch = "wasm32")]
                {
                    crate::wasm::share();
                }
            }
            ButtonInteraction::BuyMoreHints => {
                hint_state.hints_remaining += 3; //TODO actually make them buy them!
                *popup_state.as_mut() = PopupState::None;
            }
            ButtonInteraction::ClosePopups => {
                *popup_state.as_mut() = PopupState::None;
            }
        }
    }
}
