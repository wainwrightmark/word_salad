use bevy::{log::LogPlugin, window::PrimaryWindow};
use crate::animated_solutions::AnimatedSolutionPlugin;
pub use crate::prelude::*;


pub fn go() {
    let mut app = App::new();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resize_constraints: WindowResizeConstraints {
                min_height: WINDOW_HEIGHT,
                min_width: WINDOW_WIDTH,
                max_width: WINDOW_WIDTH,
                max_height: WINDOW_HEIGHT,
            },
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
    app.register_transition::<TransformRotationYLens>();

    app.add_systems(Update, handle_mouse_input);
    app.add_systems(Update, handle_touch_input);
    //app.add_systems(Update, draw_shape);
    app.add_systems(Update, button_system);
    app.insert_resource(LazyLevelData::new_empty());
    app.add_systems(First, update_lazy_level_data);

    //app.add_plugins(bevy_utils::window_size::WindowSizePlugin);

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

fn adjust_cursor_position(p: Vec2) -> Vec2 {
    Vec2 {
        x: p.x - (WINDOW_WIDTH * 0.5),
        y: (WINDOW_HEIGHT * 0.5) - p.y,
    }
}

fn handle_mouse_input(
    mouse_input: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
) {
    if mouse_input.just_released(MouseButton::Left) {
        if let Some(tile) = try_get_cursor_tile(q_windows) {
            input_state.handle_input_end(&mut chosen_state, tile);
        } else {
            input_state.handle_input_end_no_location();
        }
    } else if mouse_input.just_pressed(MouseButton::Left) {
        let Some(tile) = try_get_cursor_tile(q_windows) else {
            return;
        };
        input_state.handle_input_start(
            &mut chosen_state,
            tile,
            &current_level.level().grid,
            &found_words,
        )
    } else if mouse_input.pressed(MouseButton::Left) {
        let Some(tile) = try_get_cursor_tile(q_windows) else {
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

fn try_get_cursor_tile(q_windows: Query<&Window, With<PrimaryWindow>>) -> Option<Tile> {
    let window = q_windows.iter().next()?;

    let cursor_position = window.cursor_position()?;
    let cursor_position = adjust_cursor_position(cursor_position);
    let tile = pick_tile(cursor_position);

    Tile::try_from_dynamic(tile)
}

fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,

    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
) {
    for ev in touch_events.into_iter() {
        match ev.phase {
            bevy::input::touch::TouchPhase::Started => {
                let Some(tile) = try_get_tile(ev.position, &q_camera) else {
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
                let Some(tile) = try_get_tile(ev.position, &q_camera) else {
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
                if let Some(tile) = try_get_tile(ev.position, &q_camera) {
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

fn try_get_tile(position: Vec2, q_camera: &Query<(&Camera, &GlobalTransform)>) -> Option<Tile> {
    let position = convert_screen_to_world_position(position, q_camera)?;
    let tile = pick_tile(position);
    let tile = Tile::try_from_dynamic(tile)?;
    Some(tile)
}

fn convert_screen_to_world_position(
    screen_pos: Vec2,
    q_camera: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = q_camera.single();
    camera.viewport_to_world_2d(camera_transform, screen_pos)
}

pub fn pick_tile(position: Vec2) -> DynamicTile {
    let position = position - GRID_TOP_LEFT; // - (TILE_SIZE * 0.5);

    let dv = DynamicVertex::from_center(&position, SCALE);
    let dt = dv.get_tile(&Corner::SouthEast);
    dt
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
                    *current_level =match current_level.as_ref() {
                        CurrentLevel::Fixed { level_index } => CurrentLevel::Fixed { level_index: level_index + 1 },
                        CurrentLevel::Custom(_) => CurrentLevel::Fixed { level_index: 0 },
                    };

                    *found_words = FoundWordsState::default();
                    *chosen_state = ChosenState::default();
                }
            }
        }
    }
}

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(mut current_level: ResMut<CurrentLevel>) {
    info!("Choose level 1");
    #[cfg(target_arch = "wasm32")]
    {
        info!("Choose level 2");
        match get_game_from_location() {
            Some(level) => {
                //info!("Loaded level from url");
                *current_level = CurrentLevel::Custom(level);
                return;
            }
            None => {
                //info!("No url game to load")
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