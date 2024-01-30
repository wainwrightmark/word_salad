use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use strum::EnumIs;

const CLEAR_COLOR: Color = {
    //Color::NONE
    if cfg!(target_arch = "wasm32") {
        Color::NONE
    } else {
        Color::WHITE
    }
};

maveric::define_lens_transparent!(ClearColorLens, ClearColor, Color);

pub struct ClearColorPlugin;

impl Plugin for ClearColorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .register_resource_transition::<ClearColorLens>()
            .add_systems(Update, clear_color_transition);
    }
}

fn clear_color_transition(
    video: Res<VideoResource>,
    current_level: Res<CurrentLevel>,
    found_words: Res<FoundWordsState>,
    mut clear_transition: ResMut<ResourceTransition<ClearColorLens>>,
    clear_color: Res<ClearColor>,
) {
    if video.is_changed() || current_level.is_changed() || found_words.is_changed() {
        let background_type = BackgroundType::from_resources(&video, &current_level, &found_words);

        if clear_transition.transition.is_some() || clear_color.0 != background_type.color() {
            if background_type.is_transition_instant() {
                clear_transition.transition = Some(Transition::SetValue {
                    value: background_type.color(),
                    next: None,
                })
            } else {
                clear_transition.transition = Some(Transition::Wait {
                    remaining: Duration::from_secs_f32(crate::view::TILE_LINGER_SECONDS),
                    next: Some(Box::new(Transition::ThenEase {
                        next: None,
                        destination: background_type.color(),
                        speed: 2.0.into(),
                        ease: Ease::CubicOut,
                    })),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum BackgroundType {
    Congrats,
    NonLevel,
    Selfie,
    Normal,
}

impl BackgroundType {
    pub fn from_resources(
        video: &VideoResource,
        current_level: &CurrentLevel,
        found_words: &FoundWordsState,
    ) -> Self {
        if video.is_selfie_mode {
            Self::Selfie
        } else if current_level.is_non_level() {
            Self::NonLevel
        } else if found_words.is_level_complete() {
            Self::Congrats
        } else {
            Self::Normal
        }
    }

    pub fn color(&self) -> Color {
        match self {
            BackgroundType::Congrats => palette::CLEAR_COLOR_CONGRATS,
            BackgroundType::NonLevel => palette::CLEAR_COLOR_NON_LEVEL,
            BackgroundType::Selfie => palette::CLEAR_COLOR_SELFIE,
            BackgroundType::Normal => palette::CLEAR_COLOR_NORMAL,
        }
        .convert_color()
    }

    pub fn is_transition_instant(&self) -> bool {
        match self {
            BackgroundType::Congrats => false,
            BackgroundType::NonLevel => true,
            BackgroundType::Selfie => true,
            BackgroundType::Normal => true,
        }
    }
}
