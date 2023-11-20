use crate::constants::SaladWindowSize;
use crate::prelude::*;
use bevy::prelude::*;
use maveric::transition::speed::LinearSpeed;
use ws_core::layout::entities::*;
use ws_core::prelude::*;

pub struct AnimatedSolutionPlugin;

impl Plugin for AnimatedSolutionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_scheduled_for_removal);

        //app.register_transition::<StyleTopLens>();
    }
}

pub fn animate_solution(
    commands: &mut Commands,
    solution: &Solution,
    word: &Word,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
    level: &CurrentLevel,
) {

    //info!("Animate solution");
    let color = if is_first_time {
        Color::LIME_GREEN
    } else {
        Color::YELLOW
    };

    const SECONDS: f32 = 3.0;
    const SPACING: f32 = 0.5;
    const MID_SCALE: f32 = 0.75;
    const SPEED_MULTIPLIER: f32 = 25.0;

    let words = &level.level().words;

    let Some(layout_word_tile) = words.iter().position(|x| x == word).map(LayoutWordTile) else {
        return;
    };

    let Some(last_tile) = solution.last() else{return;};

    let mid_destination = size.get_rect(&LayoutGridTile(*last_tile), &()).centre();
    let word_destination_rect = size.get_rect(&layout_word_tile, words);
    let word_destination_centre = word_destination_rect.centre();

    //info!("Animate to {mid_destination}, then {word_destination_centre}", );

    let font = asset_server.load(SOLUTIONS_FONT_PATH);
    let font_size = size.font_size::<LayoutGridTile>();

    let scale_speed = LinearSpeed{units_per_second: 1.0 / SECONDS};
    let translation_speed = LinearSpeed{units_per_second: word_destination_rect.extents.y.abs() * SPEED_MULTIPLIER };
    let speed = (translation_speed, scale_speed);

    let right_push = ((mid_destination.x - ((solution.len() as f32 + 0.5) * font_size * SPACING)) + (size.scaled_width * 0.5)).min(0.0);


    for (index, (tile, character)) in solution.iter().zip(word.characters.iter()).enumerate() {
        let index_back = solution.len() - (index + 1);
        let text = Text::from_section(
            character.as_char().to_string(),
            TextStyle {
                font_size: font_size,
                color,
                font: font.clone(),
            },
        );
        let start_position = size.get_rect(&LayoutGridTile(*tile), &()).centre();


        let step_two = TransitionStep::<(TransformTranslationLens, TransformScaleLens)>:: new_arc(
            (word_destination_centre.extend(crate::z_indices::ANIMATED_SOLUTION), Vec3::ZERO),
            Some(speed),
            NextStep::None,
        );

        let destination_one =  mid_destination - Vec2{x: right_push + (index_back as f32 * font_size * SPACING), y: 0.0 } ;

        //let speed_one = calculate_speed(&start_position, &destination_one, core::time::Duration::from_secs_f32(2.0));

        let step_one = TransitionStep::<(TransformTranslationLens, TransformScaleLens)>::new_arc(
            (destination_one.extend(crate::z_indices::ANIMATED_SOLUTION), Vec3::ONE * MID_SCALE),
            Some(speed),

            NextStep::Step(step_two)
        );

        let components = (
            ScheduledForDeletion {
                timer: Timer::from_seconds(SECONDS, TimerMode::Once),
            },
            Text2dBundle {
                text,
                transform: Transform::from_translation(
                    start_position.extend(crate::z_indices::ANIMATED_SOLUTION),
                ),
                ..Default::default()
            },
            Transition::new(step_one)
        );

        commands.spawn(components);
    }







}

#[derive(Debug, Component)]
pub(crate) struct ScheduledForDeletion {
    pub timer: Timer,
}

#[allow(clippy::needless_pass_by_value)]
fn handle_scheduled_for_removal(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ScheduledForDeletion)>,
) {
    for (entity, mut schedule) in query.iter_mut() {
        schedule.timer.tick(time.delta());
        if schedule.timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

