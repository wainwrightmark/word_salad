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
        let new_color = get_clear_color(&video, &current_level, &found_words);

        if clear_transition.transition.is_some() || clear_color.0 != new_color {
            clear_transition.transition = Some(Transition::ThenEase {
                destination: new_color,
                speed: 1.0.into(),
                ease: Ease::CubicInOut,
                next: None,
            })
        }
    }
}

fn get_clear_color(
    video: &VideoResource,
    current_level: &CurrentLevel,
    found_words: &FoundWordsState,
) -> Color {
    if video.is_selfie_mode {
        palette::CLEAR_COLOR_SELFIE.convert_color()
    } else if current_level.is_non_level() {
        palette::CLEAR_COLOR_NON_LEVEL.convert_color()
    } else if found_words.is_level_complete() {
        palette::CLEAR_COLOR_CONGRATS.convert_color()
    } else {
        palette::CLEAR_COLOR_NORMAL.convert_color()
    }
}
