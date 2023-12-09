use std::time::Duration;

use crate::constants::SaladWindowSize;
use crate::prelude::*;
use bevy::prelude::*;
use maveric::transition::speed::calculate_speed;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

const STEP_ONE_SCALE_SECONDS: f32 = 1.0;
const STEP_ONE_TRANSLATION_SECONDS: f32 = 2.0;


pub const TOTAL_SECONDS: f32 = STEP_ONE_TRANSLATION_SECONDS;

pub fn animate_solution(
    commands: &mut Commands,
    solution: &Solution,
    word: &DisplayWord,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
    level: &CurrentLevel,
) {
    //info!("Animate solution");
    let color = if is_first_time {
        palette::ANIMATED_SOLUTION_NEW
    } else {
        palette::ANIMATED_SOLUTION_OLD
    };

    let time_multiplier = if is_first_time { 1.0 } else { 0.5 };

    const SPACING: f32 = 0.4;
    const MID_SCALE: f32 = 0.75;

    let words = &level.level().words;

    let Some(layout_word_tile) = words.iter().position(|x| x == word).map(LayoutWordTile) else {
        return;
    };

    //let Some(last_tile) = solution.last() else{return;};

    // let mid_destination = size
    //     .get_rect(&LayoutTextItem::FoundWordAnimation, &())
    //     .centre();
    let word_destination_rect = size.get_rect(&layout_word_tile, words);
    let word_destination_centre = word_destination_rect.centre();

    //info!("Animate to {mid_destination}, then {word_destination_centre}", );

    let font = asset_server.load(SOLUTIONS_FONT_PATH);
    let font_size = size.font_size::<LayoutGridTile>(&LayoutGridTile::default());

    let speed_one_scale = calculate_speed(
        &Vec3::ONE,
        &(Vec3::ONE * MID_SCALE),
        Duration::from_secs_f32(STEP_ONE_SCALE_SECONDS * time_multiplier),
    );
    // let speed_two_scale = calculate_speed(
    //     &(Vec3::ONE),
    //     &(Vec3::ONE * MID_SCALE),
    //     Duration::from_secs_f32(STEP_TWO_SCALE_SECONDS * time_multiplier),
    // );

    //let right_push = ((mid_destination.x - ((solution.len() as f32 + 0.5) * font_size * SPACING)) + (size.scaled_width * 0.5)).min(0.0);

    for (index, (tile, character)) in solution
        .iter()
        .zip(word.graphemes.iter().filter(|x| x.is_game_char))
        .enumerate()
    {
        let text = Text::from_section(
            character.grapheme.clone(),
            TextStyle {
                font_size,
                color: color.convert_color(),
                font: font.clone(),
            },
        );
        let offset = (solution.len() as f32 / 2.0) - index as f32;
        // let destination_one = mid_destination
        //     - Vec2 {
        //         x: ((offset - 0.5) * font_size * SPACING),
        //         y: 0.0,
        //     };

        let destination_two = word_destination_centre
            - Vec2 {
                x: ((offset - 0.5) * font_size * SPACING * 0.5),
                y: 0.0,
            };

        let start_position = size.get_rect(&LayoutGridTile(*tile), &()).centre();
        // let speed_two_translation = calculate_speed(
        //     &destination_two,
        //     &destination_one,
        //     Duration::from_secs_f32(STEP_TWO_TRANSLATION_SECONDS * time_multiplier),
        // );

        // let step_two = TransitionStep::<(TransformTranslationLens, TransformScaleLens)>::new_arc(
        //     (
        //         destination_two.extend(crate::z_indices::ANIMATED_SOLUTION),
        //         Vec3::ZERO,
        //     ),
        //     Some((speed_two_translation, speed_two_scale)),
        //     NextStep::None,
        // );

        let speed_one_translation = calculate_speed(
            &start_position,
            &destination_two,
            core::time::Duration::from_secs_f32(STEP_ONE_TRANSLATION_SECONDS * time_multiplier),
        );

        let step_one = TransitionStep::<(TransformTranslationLens, TransformScaleLens)>::new_arc(
            (
                destination_two.extend(crate::z_indices::ANIMATED_SOLUTION),
                Vec3::ONE * MID_SCALE,
            ),
            Some((speed_one_translation, speed_one_scale)),
            NextStep::None,
        );

        let components = (
            ScheduledForDeletion {
                timer: Timer::from_seconds(TOTAL_SECONDS * time_multiplier, TimerMode::Once),
            },
            Text2dBundle {
                text,
                transform: Transform::from_translation(
                    start_position.extend(crate::z_indices::ANIMATED_SOLUTION),
                ),
                ..Default::default()
            },
            Transition::new(step_one),
        );

        commands.spawn(components);
    }
}
