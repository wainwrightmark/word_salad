use std::time::Duration;

use crate::constants::SaladWindowSize;
use crate::motion_blur::MotionBlur;
use crate::prelude::*;
use bevy::prelude::*;
use maveric::transition::speed::calculate_speed;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

const STEP_ONE_SCALE_SECONDS: f32 = 1.0;
const STEP_ONE_TRANSLATION_SECONDS: f32 = 1.5;

pub const TOTAL_SECONDS: f32 = STEP_ONE_TRANSLATION_SECONDS;

#[derive(Debug, Event)]
pub struct WordFoundEvent {
    pub solution: Solution,
    pub is_first_time: bool,
    pub was_hinted: bool,
    pub word: DisplayWord,
    pub level: DesignedLevel,
}

pub fn animate_solutions(
    mut commands: Commands,
    mut events: EventReader<WordFoundEvent>,
    asset_server: Res<AssetServer>,
    size: Res<Size>,
    video: Res<VideoResource>,
) {
    for ev in events.read() {
        animate_solution(
            &mut commands,
            &ev.solution,
            &ev.word,
            ev.is_first_time,
            asset_server.as_ref(),
            size.as_ref(),
            &ev.level,
            video.selfie_mode(),
        );
    }
}

pub fn remove_animated_solutions_on_complete(
    found_words: Res<FoundWordsState>,
    mut commands: Commands,
    solutions: Query<Entity, With<AnimatedSolutionMarker>>,
) {
    if found_words.is_changed() && found_words.is_level_complete() {
        for entity in solutions.into_iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Debug, Component, Clone, Copy, PartialEq)]
pub struct AnimatedSolutionMarker;

fn animate_solution(
    commands: &mut Commands,
    solution: &Solution,
    word: &DisplayWord,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
    level: &DesignedLevel,
    selfie_mode: SelfieMode,
) {
    //info!("Animate solution");
    let color = if is_first_time {
        palette::ANIMATED_SOLUTION_NEW
    } else {
        palette::ANIMATED_SOLUTION_OLD
    };

    let time_multiplier = if is_first_time { 1.0 } else { 0.5 };

    const SPACING: f32 = 0.4;
    const MID_SCALE: f32 = 0.5;

    let words = level.words.as_slice();

    let Some(layout_word_tile) = words.iter().position(|x| x == word).map(LayoutWordTile) else {
        return;
    };
    let word_destination_rect = size.get_rect(&layout_word_tile, &(words, selfie_mode));
    let word_destination_centre = word_destination_rect.centre();

    let font = asset_server.load(SOLUTIONS_FONT_PATH);
    let font_size = size.font_size::<LayoutGridTile>(&LayoutGridTile::default(), &());

    let speed_one_scale = calculate_speed(
        &Vec3::ONE,
        &(Vec3::ONE * MID_SCALE),
        Duration::from_secs_f32(STEP_ONE_SCALE_SECONDS * time_multiplier),
    );

    for (index, (tile, character)) in solution
        .iter()
        .zip(word.graphemes.iter().filter(|x| x.is_game_char))
        .enumerate()
    {
        let text = Text::from_section(
            character.grapheme.to_uppercase(),
            TextStyle {
                font_size,
                color: color.convert_color(),
                font: font.clone(),
            },
        );
        let offset = (solution.len() as f32 / 2.0) - index as f32;

        let destination_two = word_destination_centre
            - Vec2 {
                x: ((offset - 0.5) * font_size * SPACING * 0.5),
                y: 0.0,
            };

        let start_position = size.get_rect(&LayoutGridTile(*tile), &selfie_mode).centre();

        let speed_one_translation = calculate_speed(
            &start_position,
            &destination_two,
            core::time::Duration::from_secs_f32(STEP_ONE_TRANSLATION_SECONDS * time_multiplier),
        );

        let transition =
            TransitionBuilder::<(TransformTranslationLens, TransformScaleLens)>::default()
                .then_ease(
                    (
                        destination_two.extend(crate::z_indices::ANIMATED_SOLUTION),
                        Vec3::ONE * MID_SCALE,
                    ),
                    (speed_one_translation, speed_one_scale),
                    Ease::SineOut,
                )
                .build();

        let components = (
            ScheduledForDeletion {
                remaining: Duration::from_secs_f32(TOTAL_SECONDS * time_multiplier),
            },
            Text2dBundle {
                text: text.clone(),
                transform: Transform::from_translation(
                    start_position.extend(crate::z_indices::ANIMATED_SOLUTION),
                ),
                ..Default::default()
            },
            transition,
            AnimatedSolutionMarker,
        );

        let parent_entity = commands.spawn(components).id();

        //info!("{speed_one_translation}");

        let mut scale = speed_one_translation.units_per_second / 300.0; //300.0; //scale the amount of blur with the speed
        let mut a = 0.6; //300.0; //scale the amount of blur with the speed
        for frame_offset in 1..=3 {
            let mut text = text.clone();
            for section in text.sections.iter_mut() {
                section.style.color = section.style.color.with_a(a);
            }
            a *= 0.8;
            scale *= 0.8;

            commands.spawn((
                Text2dBundle {
                    text,
                    transform: Transform::from_scale(Vec3::ONE * scale),
                    ..Default::default()
                },
                MotionBlur::new(frame_offset * 2, parent_entity),
                AnimatedSolutionMarker,
            ));
        }
    }
}
