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
        let background_type = background_type_from_resources(&video, &current_level, &found_words);

        if clear_transition.transition.is_some()
            || clear_color.0 != background_type.clear_color().convert_color()
        {
            let is_instant = current_level.is_changed() || !background_type.is_congrats();

            if is_instant {
                clear_transition.transition = Some(Transition::SetValue {
                    value: background_type.clear_color().convert_color(),
                    next: None,
                })
            } else {
                const WAIT_SECS: f32 = 1.0;
                const TRANSITION_SECS: f32 = 2.0;

                let transition = TransitionBuilder::default()
                    .then_set_value(clear_color.0)
                    .then_wait(Duration::from_secs_f32(WAIT_SECS))
                    .then_ease_with_duration(
                        background_type.clear_color().convert_color(),
                        Duration::from_secs_f32(TRANSITION_SECS),
                        Ease::CubicOut,
                    )
                    .build();

                clear_transition.transition = Some(transition);
            }
        }
    }
}

pub fn background_type_from_resources(
    video: &VideoResource,
    current_level: &CurrentLevel,
    found_words: &FoundWordsState,
) -> BackgroundType {
    if video.is_selfie_mode {
        BackgroundType::Selfie
    } else if current_level.is_non_level() {
        BackgroundType::NonLevel
    } else if found_words.is_level_complete() {
        BackgroundType::Congrats
    } else {
        BackgroundType::Normal
    }
}
