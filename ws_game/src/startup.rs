pub use crate::prelude::*;
use bevy::{log::LogPlugin, window::PrimaryWindow};
use bevy_utils::window_size::WindowSizePlugin;

const CLEAR_COLOR : Color ={
    if cfg!(target_arch = "wasm32"){
        Color::NONE
    }
    else{
        Color::ALICE_BLUE
    }
};

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
        .insert_resource(ClearColor(CLEAR_COLOR))
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
    app.register_transition::<TransformScaleLens>();

    app.add_systems(Update, handle_mouse_input);
    app.add_systems(Update, handle_touch_input);
    //app.add_systems(Update, draw_shape);
    app.add_plugins(MenuPlugin);
    app.insert_resource(LazyLevelData::new_empty());
    app.add_systems(First, update_lazy_level_data);

    app.add_plugins(WindowSizePlugin::<SaladWindowBreakPoints>::default());

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(crate::wasm::WasmPlugin);
    app.add_plugins(VideoPlugin);



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



const MOVE_TOLERANCE: f32 = 0.3;

fn handle_mouse_input(
    menu_state: Res<MenuState>,
    mouse_input: Res<Input<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
    size: Res<Size>,
) {
    if !menu_state.is_closed() {
        return;
    }

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
    tolerance: Option<f32>,
) -> Option<Tile> {
    let window = q_windows.iter().next()?;

    let cursor_position = window.cursor_position()?;
    let cursor_position = size.adjust_cursor_position(cursor_position);
    let tile = match tolerance {
        Some(tolerance) => size.try_pick_tile(cursor_position, tolerance)?,
        None => size.pick_tile(cursor_position),
    };

    Tile::try_from_dynamic(tile)
}

fn handle_touch_input(
    menu_state: Res<MenuState>,
    mut touch_events: EventReader<TouchInput>,

    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut chosen_state: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut input_state: Local<InputState>,
    found_words: Res<FoundWordsState>,
    size: Res<Size>,
) {
    if !menu_state.is_closed() {
        return;
    }

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
                let Some(tile) = try_get_tile(ev.position, &q_camera, &size, Some(MOVE_TOLERANCE))
                else {
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
    tolerance: Option<f32>,
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

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(
    mut current_level: ResMut<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    mut chosen_state: ResMut<ChosenState>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                *current_level = CurrentLevel::Custom(level);
                *found_words = FoundWordsState::default();
                *chosen_state = ChosenState::default();
                return;
            }
            None => {}
        }
    }
}

