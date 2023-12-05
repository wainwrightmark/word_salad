use crate::input::InputPlugin;
pub use crate::prelude::*;
use bevy::log::LogPlugin;
use nice_bevy_utils::window_size::WindowSizePlugin;
use ws_core::layout::entities::*;

const CLEAR_COLOR: Color = {
    if cfg!(target_arch = "wasm32") {
        Color::NONE
    } else {
        crate::prelude::convert_color_const(palette::GAME_BACKGROUND)
    }
};

pub fn go() {
    let mut app = App::new();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "Word Salad".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::new(IDEAL_WIDTH, IDEAL_HEIGHT),
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

    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(CLEAR_COLOR))
        .add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(log_plugin)
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(
                    bevy_embedded_assets::EmbeddedAssetPlugin {
                        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
                    },
                ),
        )
        .add_systems(Startup, setup_system);

    app.register_maveric::<ViewRoot>();
    app.add_plugins(StatePlugin);
    app.add_plugins(LevelTimePlugin);
    app.add_plugins(ShapesPlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(LogWatchPlugin);
    app.add_plugins(BackgroundPlugin);

    app.register_transition::<BackgroundColorLens>();
    app.register_transition::<TransformRotationYLens>();
    app.register_transition::<TransformTranslationLens>();
    app.register_transition::<TransformScaleLens>();
    app.register_transition::<(TransformTranslationLens, TransformScaleLens)>();

    app.add_plugins(InputPlugin);

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

    // #[cfg(debug_assertions)]
    // {
    //     app.add_systems(Update, draw_gizmos);
    // }

    app.run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// fn draw_gizmos(size: Res<Size>, mut gizmos: Gizmos) {
//     let grid_rect = size.get_rect(LayoutEntity::Grid);

//     gizmos.rect_2d(grid_rect.centre(), 0.0, grid_rect.extents, Color::RED);
// }

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(
    mut current_level: ResMut<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    mut chosen_state: ResMut<ChosenState>,
    mut timer: ResMut<crate::level_time::LevelTime>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                *current_level = CurrentLevel::Custom(level);
                *found_words = FoundWordsState::new_from_level(current_level.as_ref());
                *chosen_state = ChosenState::default();
                *timer = LevelTime::default();
                return;
            }
            None => {}
        }
    }
}
