use crate::constants::SaladWindowSize;
use crate::prelude::*;
use bevy::prelude::*;
use maveric::transition::speed::calculate_speed;
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
    last_tile: Tile,
    word: &Word,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
    level: &CurrentLevel,
) {
    let color = if is_first_time {
        Color::LIME_GREEN
    } else {
        Color::YELLOW
    };

    const SECONDS: f32 = 2.0;
    let words = &level.level().words;

    let Some(layout_word_tile) = words.iter().position(|x| x == word).map(LayoutWordTile) else {
        return;
    };

    let start_position = size.get_rect(&LayoutGridTile(last_tile), &()).centre();

    let destination = size.get_rect(&layout_word_tile, words).centre();

    let speed = calculate_speed(
        &start_position,
        &destination,
        std::time::Duration::from_secs_f32(SECONDS),
    );

    let font = asset_server.load(SOLUTIONS_FONT_PATH);

    let text = Text::from_section(
        word.text.clone(),
        TextStyle {
            font_size: 32.0,
            color,
            font,
        },
    );

    let components = (
        ScheduledForDeletion {
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        },
        Text2dBundle {
            text,
            transform: Transform::from_translation(
                start_position.extend(crate::z_indices::ANIMATED_SOLUTION),
            ),
            ..Default::default()
        },
        Transition::<TransformTranslationLens>::new(TransitionStep::new_arc(
            destination.extend(crate::z_indices::ANIMATED_SOLUTION),
            Some(speed),
            NextStep::None,
        )),
    );

    commands.spawn(components);
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
