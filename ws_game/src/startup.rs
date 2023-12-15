use crate::input::InputPlugin;
pub use crate::prelude::*;
use bevy::log::LogPlugin;
use nice_bevy_utils::{async_event_writer, window_size::WindowSizePlugin, CanRegisterAsyncEvent};
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
    app.add_plugins(DailyChallengePlugin);

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

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.insert_resource(bevy_pkv::PkvStore::new_in_dir("saves"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        app.insert_resource(bevy_pkv::PkvStore::new("bleppo", "word_salad"));
    }

    app.add_systems(Startup, hide_splash);

    app.register_async_event::<AppLifeCycleEvent>();
    app.add_systems(Startup, begin_lifecycle);
    app.add_systems(Update, watch_lifecycle);
    app.add_systems(Startup, set_status_bar.after(hide_splash));

    app.insert_resource(bevy::winit::WinitSettings {
        return_from_run: false,
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::Reactive {
            wait: std::time::Duration::from_secs(60),
        },
    });

    app.run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(
    mut current_level: ResMut<CurrentLevel>,
    mut timer: ResMut<crate::level_time::LevelTime>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                if let Err(err) = CUSTOM_LEVEL.set(level){
                    error!("{err}");
                }
                *current_level = CurrentLevel::Custom;
                *timer = LevelTime::default();
                return;
            }
            None => {}
        }
    }
}

fn hide_splash() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        do_or_report_error(capacitor_bindings::splash_screen::SplashScreen::hide(
            2000.0,
        ));
    }
}

fn set_status_bar() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        use capacitor_bindings::status_bar::*;

        do_or_report_error(StatusBar::set_style(Style::Dark));
        #[cfg(feature = "android")]
        do_or_report_error(StatusBar::set_background_color("#5B8BE2"));
    }
}

fn watch_lifecycle(
    mut events: EventReader<AppLifeCycleEvent>,
    mut video: ResMut<VideoResource>,
    mut menu: ResMut<MenuState>,
) {
    for event in events.read() {
        match event {
            AppLifeCycleEvent::StateChange { is_active } => {
                //info!("State change is_active {is_active}");
                if *is_active && video.is_selfie_mode {
                    video.is_selfie_mode = false;
                }
            }
            AppLifeCycleEvent::BackPressed => {
                //info!("State change Back Pressed");
                if !menu.is_closed() {
                    menu.close();
                }
            }
        }
    }
}

fn begin_lifecycle(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    spawn_and_run(disable_back_async(writer.clone()));
    spawn_and_run(on_resume(writer));
}

async fn disable_back_async<'a>(_writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(feature = "android")]
    {
        info!("Disabling back");
        let result = capacitor_bindings::app::App::add_back_button_listener(move |_| {
            //info!("Sending back event");
            _writer
                .send_blocking(AppLifeCycleEvent::BackPressed)
                .unwrap();
        })
        .await;

        match result {
            Ok(handle) => {
                //info!("Leading back button");
                handle.leak();
            }
            Err(err) => {
                crate::logging::try_log_error_message(format!("{err}"));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Event, PartialEq)]
enum AppLifeCycleEvent {
    StateChange { is_active: bool },
    BackPressed,
}

async fn on_resume(_writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        //info!("Setting on_resume");
        let result = capacitor_bindings::app::App::add_state_change_listener(move |x| {
            _writer
                .send_blocking(AppLifeCycleEvent::StateChange {
                    is_active: x.is_active,
                })
                .unwrap();
        })
        .await;

        match result {
            Ok(handle) => {
                handle.leak();
            }
            Err(err) => {
                crate::logging::try_log_error_message(format!("{err}"));
            }
        }
    }
}
