use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;

const CLEAR_COLOR: Color = {
    //Color::NONE
    if cfg!(target_arch = "wasm32") {
        Color::NONE
    } else {
        Color::WHITE
    }
};

define_lens_transparent!(ClearColorLens, ClearColor, Color);

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
        let ClearColorTransition { color, instant } = get_clear_color_transition(&video, &current_level, &found_words);

        if clear_transition.transition.is_some() || clear_color.0 != color {

            if instant{
                clear_transition.transition = Some(
                    Transition::SetValue { value: color, next: None },
                )
            }else{
                clear_transition.transition = Some(
                    Transition::Wait {
                        remaining: Duration::from_secs_f32(crate::view::TILE_LINGER_SECONDS),
                        next: Some(Box::new(Transition::ThenEase {

                            next: None,
                            destination: color,
                            speed: 2.0.into(),
                            ease: Ease::CubicOut,
                        })),
                    },
                )
            }



        }
    }
}

struct ClearColorTransition{
    pub color: Color,
    pub instant: bool
}

fn get_clear_color_transition(
    video: &VideoResource,
    current_level: &CurrentLevel,
    found_words: &FoundWordsState,
) -> ClearColorTransition {
    if video.is_selfie_mode {
        ClearColorTransition{color:palette::CLEAR_COLOR_SELFIE.convert_color(), instant: true }
    } else if current_level.is_non_level() {
        ClearColorTransition{color: palette::CLEAR_COLOR_NON_LEVEL.convert_color(), instant: true }

    } else if found_words.is_level_complete() {
        ClearColorTransition{color: palette::CLEAR_COLOR_CONGRATS.convert_color(), instant: false }

    } else {
        ClearColorTransition{color: palette::CLEAR_COLOR_NORMAL.convert_color(), instant: true }

    }
}
