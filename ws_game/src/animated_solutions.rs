use crate::constants::SaladWindowSize;
use crate::prelude::*;
use bevy::prelude::*;
use maveric::transition::speed::calculate_speed;
use ws_core::Tile;

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
    word: &Word ,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
    level: &CurrentLevel
) {
    let color = if is_first_time {
        Color::LIME_GREEN
    } else {
        Color::YELLOW
    };

    const SECONDS: f32 = 2.0;

    let Some(word_index) = level.level().words.iter().position(|x|x == word).and_then(|x| WordTile::try_from_usize(x)) else {return;};

    let start_position = size.get_rect(&LayoutGridTile(last_tile)).centre();

    let destination = size.get_rect(&LayoutWordTile(word_index)).centre();

    let speed = calculate_speed(
        &start_position,
        &destination,
        std::time::Duration::from_secs_f32(SECONDS),
    );

    let font = get_or_load_asset(SOLUTIONS_FONT_PATH, asset_server);

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
            transform: Transform::from_translation(start_position.extend(crate::z_indices::ANIMATED_SOLUTION)),
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

pub(crate) fn get_or_load_asset<T: bevy::asset::Asset>(
    path: &str,
    server: &AssetServer,
) -> Handle<T> {
    let asset: Handle<T> = match server.get_load_state(path) {
        bevy::asset::LoadState::Loaded => server.get_handle(path),
        _ => server.load(path),
    };
    asset
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
