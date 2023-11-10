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
    word: &Word,
    is_first_time: bool,
    asset_server: &AssetServer,
    size: &Size,
) {
    let color = if is_first_time {
        Color::LIME_GREEN
    } else {
        Color::YELLOW
    };

    const SECONDS: f32 = 2.0;

    // let mut tile_location = last_tile.get_center(size.scale());
    // tile_location.y = WINDOW_HEIGHT / 2.0 - tile_location.y;
    // tile_location.x = tile_location.x   - (size.tile_size() * 0.5);

    // let location = size.grid_top_left() + tile_location ;

    let start_position = size.tile_position(&last_tile);
    let destination = Vec2 {
        x: 0.0,
        y: size.scaled_height * -1.0,
    };

    //bevy::log::info!("tile: {last_tile}. location {location}");

    let speed = calculate_speed(
        &start_position,
        &destination,
        std::time::Duration::from_secs_f32(SECONDS),
    );

    let font = get_or_load_asset(FONT_PATH, asset_server);

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
            transform: Transform::from_translation(start_position.extend(0.0)),
            ..Default::default()
        },
        Transition::<TransformTranslationLens>::new(TransitionStep::new_arc(
            destination.extend(0.0),
            Some(speed),
            NextStep::None,
        )), // Transition::<StyleTopLens>::new(TransitionStep::new_arc(
            //     Val::Px(destination.y),
            //     Some(speed),
            //     NextStep::None,
            // )),
            // TextBundle {

            //     text,
            //     style: Style {
            //         position_type: PositionType::Absolute,
            //         top: Val::Px(start_position.y ),
            //         left: Val::Px(start_position.x + (WINDOW_WIDTH / 2.0)),
            //         ..default()
            //     },
            //     ..Default::default()
            // },
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
