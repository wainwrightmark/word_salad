pub use crate::prelude::*;
use crate::{completion::TotalCompletion, input::InputPlugin, motion_blur::MotionBlurPlugin};
use bevy::log::LogPlugin;
use itertools::Either;
use nice_bevy_utils::{async_event_writer, window_size::WindowSizePlugin, CanRegisterAsyncEvent};
use ws_core::layout::entities::*;

const CLEAR_COLOR: Color = {
    //Color::NONE
    if cfg!(target_arch = "wasm32") {
        Color::NONE
    } else {
        Color::WHITE
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
    app.add_plugins(DailyChallengePlugin);
    app.add_plugins(StreakPlugin);
    app.add_plugins(MotionBlurPlugin);

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

    // app.add_systems(Last, stack_frames);
    // app.init_resource::<FrameStack>();

    app.insert_resource(bevy::winit::WinitSettings {
        return_from_run: false,
        focused_mode: bevy::winit::UpdateMode::Continuous,
        // focused_mode: bevy::winit::UpdateMode::Reactive {
        //     wait: std::time::Duration::from_secs(1),
        // },
        unfocused_mode: bevy::winit::UpdateMode::Reactive {
            wait: std::time::Duration::from_secs(60),
        },
    });

    app.run();
}

// #[derive(Debug, Resource)]
// pub struct FrameStack {
//     pub remaining: u16,
// }

// impl Default for FrameStack {
//     fn default() -> Self {
//         Self { remaining: 360 }
//     }
// }

// fn stack_frames(
//     mouse: Res<Input<MouseButton>>,
//     mut touch_events: EventReader<TouchInput>,
//     mut frames: ResMut<FrameStack>,
//     mut events: EventWriter<RequestRedraw>,
// ) {
//     if !touch_events.is_empty() {
//         touch_events.clear();
//         frames.remaining = FrameStack::default().remaining;
//     } else if mouse.is_changed() {
//         frames.remaining = FrameStack::default().remaining;
//     }

//     if let Some(new_remaining) = frames.remaining.checked_sub(1) {
//         frames.remaining = new_remaining;
//         events.send(RequestRedraw);
//     }
// }

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(
    mut current_level: ResMut<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    mut timer: ResMut<crate::level_time::LevelTime>,
    completion: Res<TotalCompletion>,
    daily_challenges: Res<DailyChallenges>,
) {

    #[cfg(target_arch = "wasm32")]
    {
        match crate::wasm::get_game_from_location() {
            Some(level) => {
                info!("Loaded custom level from path");
                if !current_level.as_ref().eq(&CurrentLevel::Custom {
                    name: level.name.to_string(),
                }) {
                    *current_level = CurrentLevel::Custom {
                        name: level.name.to_string(),
                    };
                    *found_words = FoundWordsState::new_from_level(&level);
                }

                if let Err(err) = CUSTOM_LEVEL.set(level) {
                    error!("{err}");
                }

                *timer = LevelTime::default();
                return;
            }
            None => {}
        }
    }

    if !found_words.is_level_complete() {
        return;
    }

    match current_level.as_ref() {
        CurrentLevel::NonLevel(NonLevel::BeforeTutorial) => {
            return;
        }

        CurrentLevel::Tutorial { .. }
        | CurrentLevel::DailyChallenge { .. }
        | CurrentLevel::NonLevel(_)
        | CurrentLevel::Custom { .. } => {
            if let Some(index) = completion.get_next_incomplete_daily_challenge_from_today() {
                let today_level = CurrentLevel::DailyChallenge { index };
                if today_level.level(&daily_challenges).is_left() {
                    //Only change to this level if we have loaded it already
                    *current_level = today_level;
                }
            } else {
                *current_level = CurrentLevel::NonLevel(NonLevel::NoMoreDailyChallenge);
            }
        }
        CurrentLevel::Fixed { sequence, .. } => {
            if let Some(level_index) = completion.get_next_level_index(*sequence) {
                *current_level = CurrentLevel::Fixed {
                    level_index,
                    sequence: *sequence,
                };
            } else {
                *current_level = CurrentLevel::NonLevel(NonLevel::NoMoreLevelSequence(*sequence));
            }
        }
    }

    if let Either::Left(dl) = current_level.level(daily_challenges.as_ref()) {
        *found_words = FoundWordsState::new_from_level(dl);
    }
    *timer = LevelTime::default();
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
    mut popup: ResMut<PopupState>,
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
                if popup.is_buy_more_hints() {
                    *popup.as_mut() = PopupState::None;
                } else if !menu.is_closed() {
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

#[allow(dead_code)]
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
