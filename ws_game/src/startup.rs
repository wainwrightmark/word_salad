use crate::animated_solutions::AnimatedSolutionPlugin;
pub use crate::prelude::*;
use bevy::{log::LogPlugin, window::PrimaryWindow};
use bevy_utils::window_size::WindowSizePlugin;

pub fn go() {
    let mut app = App::new();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::default(),
            resize_constraints: WindowResizeConstraints::default(),
            present_mode: bevy::window::PresentMode::default(),

            resizable: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let log_plugin = LogPlugin {
        level: bevy::log::Level::INFO,
        ..Default::default()
    };

    app.insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(log_plugin)
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(
                    bevy_embedded_assets::EmbeddedAssetPlugin,
                ),
        )
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_system);

    app.register_maveric::<ViewRoot>();
    app.add_plugins(StatePlugin);
    app.add_plugins(AnimatedSolutionPlugin);

    app.register_transition::<FillColorLens>();
    app.register_transition::<BackgroundColorLens>();
    app.register_transition::<TransformRotationYLens>();
    app.register_transition::<TransformTranslationLens>();

    app.add_systems(Update, handle_mouse_input);
    app.add_systems(Update, handle_touch_input);
    //app.add_systems(Update, draw_shape);
    app.add_systems(Update, button_system);
    app.insert_resource(LazyLevelData::new_empty());
    app.add_systems(First, update_lazy_level_data);

    app.add_plugins(WindowSizePlugin::<SaladWindowBreakPoints>::default());

    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, resizer);

    app.add_systems(PostStartup, choose_level_on_game_load);
    #[cfg(feature = "steam")]
    {
        app.insert_resource(bevy_pkv::PkvStore::new_in_dir("saves"));
    }
    #[cfg(not(feature = "steam"))]
    {
        app.insert_resource(bevy_pkv::PkvStore::new("bleppo", "word_salad"));
    }

    app.run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct LastSize {
    pub width: f32,
    pub height: f32,
}

#[cfg(target_arch = "wasm32")]
fn resizer( //TODO move to nice bevy utils
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut window_resized_events: EventWriter<WindowResized>,
    mut last_size: Local<LastSize>,
) {
    let window = web_sys::window().expect("no global `window` exists");
    let mut width: f32 = window.inner_width().unwrap().as_f64().unwrap() as f32;
    let mut height: f32 = window.inner_height().unwrap().as_f64().unwrap() as f32;
    if width != last_size.width || height != last_size.height {
        if let Ok((window_entity, mut window)) = windows.get_single_mut() {
            *last_size = LastSize { width, height };

            let constraints = window.resize_constraints;

            width = width.clamp(constraints.min_width, constraints.max_width);
            height = height.clamp(constraints.min_height, constraints.max_height);

            let p_width = width * window.scale_factor() as f32;
            let p_height = height * window.scale_factor() as f32;
            window
                .resolution
                .set_physical_resolution(p_width.floor() as u32, p_height.floor() as u32);
            window_resized_events.send(WindowResized {
                window: window_entity,
                height,
                width,
            });

            debug!(
                "Resizing to {:?},{:?} with scale factor of {}",
                width,
                height,
                window.scale_factor()
            );
        }
    }
}

const MOVE_TOLERANCE: f32 = 0.3;

fn handle_mouse_input(
    mouse_input: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
    size: Res<Size>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(tile) = try_get_cursor_tile(q_windows, &size, None) {
            input_state.handle_input_end(&mut chosen_state, tile);
        } else {
            input_state.handle_input_end_no_location();
        }
    } else if mouse_input.just_pressed(MouseButton::Left) {
        let Some(tile) = try_get_cursor_tile(q_windows, &size, None) else {
            return;
        };
        input_state.handle_input_start(
            &mut chosen_state,
            tile,
            &current_level.level().grid,
            &found_words,
        )
    } else if mouse_input.pressed(MouseButton::Left) {
        let Some(tile) = try_get_cursor_tile(q_windows, &size, Some(MOVE_TOLERANCE)) else {
            return;
        };
        input_state.handle_input_move(
            &mut chosen_state,
            tile,
            &current_level.level().grid,
            &found_words,
        )
    }
}

fn try_get_cursor_tile(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    size: &Size,
    tolerance: Option<f32>
) -> Option<Tile> {
    let window = q_windows.iter().next()?;

    let cursor_position = window.cursor_position()?;
    let cursor_position =  size.adjust_cursor_position(cursor_position);
    let tile = match tolerance {
        Some(tolerance) => size.try_pick_tile(cursor_position, tolerance)?,
        None => size.pick_tile(cursor_position),
    };


    Tile::try_from_dynamic(tile)
}

fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,

    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
    size: Res<Size>,
) {
    for ev in touch_events.into_iter() {
        match ev.phase {
            bevy::input::touch::TouchPhase::Started => {
                let Some(tile) = try_get_tile(ev.position, &q_camera, &size, None) else {
                    continue;
                };
                input_state.handle_input_start(
                    &mut chosen_state,
                    tile,
                    &current_level.level().grid,
                    &found_words,
                );
            }
            bevy::input::touch::TouchPhase::Moved => {
                let Some(tile) = try_get_tile(ev.position, &q_camera, &size, Some(MOVE_TOLERANCE)) else {
                    continue;
                };
                input_state.handle_input_move(
                    &mut chosen_state,
                    tile,
                    &current_level.level().grid,
                    &found_words,
                );
            }
            bevy::input::touch::TouchPhase::Ended => {
                if let Some(tile) = try_get_tile(ev.position, &q_camera, &size, None) {
                    input_state.handle_input_end(&mut chosen_state, tile);
                } else {
                    input_state.handle_input_end_no_location();
                }
            }
            bevy::input::touch::TouchPhase::Canceled => {
                input_state.handle_input_end_no_location();
            }
        }
    }
}

fn try_get_tile(
    position: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
    size: &Size,
    tolerance: Option<f32>
) -> Option<Tile> {
    let position = convert_screen_to_world_position(position, q_camera)?;


    let tile = match tolerance {
        Some(tolerance) => size.try_pick_tile(position, tolerance)?,
        None => size.pick_tile(position),
    };

    let tile = Tile::try_from_dynamic(tile)?;

    //info!("Got tile {tile} from window size {size:?}");
    Some(tile)
}

fn convert_screen_to_world_position(
    screen_pos: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    camera.viewport_to_world_2d(camera_transform, screen_pos)
}

fn button_system(
    mut interaction_query: Query<(&Interaction, &ButtonMarker), Changed<Interaction>>,
    mut current_level: ResMut<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    mut chosen_state: ResMut<ChosenState>,
) {
    for (interaction, button_marker) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match button_marker {
                ButtonMarker::Reset => {
                    current_level.set_changed();
                    *found_words = FoundWordsState::default();
                    *chosen_state = ChosenState::default();
                }
                ButtonMarker::NextLevel => {
                    *current_level = match current_level.as_ref() {
                        CurrentLevel::Fixed { level_index } => CurrentLevel::Fixed {
                            level_index: level_index + 1,
                        },
                        CurrentLevel::Custom(_) => CurrentLevel::Fixed { level_index: 0 },
                    };

                    *found_words = FoundWordsState::default();
                    *chosen_state = ChosenState::default();
                }
                ButtonMarker::Hint => {
                    found_words.try_hint(current_level.as_ref());
                },

            }
        }
    }
}

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(mut current_level: ResMut<CurrentLevel>) {

    #[cfg(target_arch = "wasm32")]
    {
        match get_game_from_location() {
            Some(level) => {
                *current_level = CurrentLevel::Custom(level);
                return;
            }
            None => {
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_game_from_location() -> Option<DesignedLevel> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    DesignedLevel::try_from_path(path)
}
