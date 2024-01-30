use std::sync::atomic::AtomicUsize;

pub use crate::prelude::*;
use crate::{input::InputPlugin, motion_blur::MotionBlurPlugin, purchases::PurchasesPlugin};
use bevy::{log::LogPlugin, window::RequestRedraw};
use nice_bevy_utils::{async_event_writer, window_size::WindowSizePlugin, CanRegisterAsyncEvent};
use ws_core::layout::entities::*;

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

    app.add_plugins(ClearColorPlugin);

    app.add_plugins(WordlinePlugin);
    app.register_maveric::<ViewRoot>();
    app.register_maveric::<HintsRemainingRoot>();
    app.add_plugins(StatePlugin);
    app.add_plugins(LevelTimePlugin);
    app.add_plugins(ShapesPlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(LogWatchPlugin);
    app.add_plugins(DailyChallengePlugin);
    app.add_plugins(StreakPlugin);
    app.add_plugins(MotionBlurPlugin);
    app.add_plugins(WordsPlugin);
    app.add_plugins(PurchasesPlugin);
    #[cfg(any(feature = "ios", feature = "android"))]
    app.add_plugins(NotificationPlugin);

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
        focused_mode: bevy::winit::UpdateMode::Reactive {
            wait: std::time::Duration::from_secs(1),
        },
        //focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
            wait: std::time::Duration::from_secs(60),
        },
    });

    //app.add_systems(PostUpdate, print_maveric_tracking);
    app.add_systems(PostUpdate, maybe_request_redraw);

    app.run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub(crate) static ADDITIONAL_TRACKING: AtomicUsize = AtomicUsize::new(0);

fn maybe_request_redraw(mut writer: EventWriter<RequestRedraw>, mut buffer: Local<bool>) {
    let should_redraw = maveric::tracing::count_transitions() > 0
        || maveric::tracing::count_graph_updates() > 0
        || maveric::tracing::count_scheduled_deletions() > 0
        || maveric::tracing::count_scheduled_changes() > 0
        || ADDITIONAL_TRACKING.load(std::sync::atomic::Ordering::SeqCst) > 0;

    if should_redraw {
        writer.send(RequestRedraw);
    }

    *buffer = should_redraw;

    ADDITIONAL_TRACKING.store(0, std::sync::atomic::Ordering::SeqCst)
}

#[allow(unused_variables, unused_mut)]
fn choose_level_on_game_load(
    current_level: Res<CurrentLevel>,
    daily_challenge_completion: Res<DailyChallengeCompletion>,
    daily_challenges: Res<DailyChallenges>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    fn get_new_level(
        current_level: Res<CurrentLevel>,
        daily_challenge_completion: Res<DailyChallengeCompletion>,
        daily_challenges: Res<DailyChallenges>,
    ) -> Option<CurrentLevel> {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(daily_index) = crate::wasm::get_daily_from_location() {
                info!("Loaded daily challenge {daily_index} from path");

                let new_level = CurrentLevel::DailyChallenge { index: daily_index };
                if new_level != *current_level {
                    //todo save progress if on an existing puzzle
                    if let Some(level) = new_level.level(&daily_challenges).left() {
                        return Some(new_level);
                    } else {
                        return Some(CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished));
                    }
                }
            }

            if let Some(level) = crate::wasm::get_game_from_location() {
                info!("Loaded custom level from path");

                let custom_level = CurrentLevel::Custom {
                    name: level.clone().full_name().to_string(),
                };

                if let Err(err) = CUSTOM_LEVEL.set(level) {
                    error!("{err}");
                }

                if !current_level.as_ref().eq(&custom_level) {
                    return Some(custom_level);
                }
            }
        }

        match current_level.as_ref() {
            CurrentLevel::Tutorial { .. } | CurrentLevel::NonLevel(NonLevel::BeforeTutorial) => {
                return None;
            }
            _ => {}
        }

        if let Some(index) =
            daily_challenge_completion.get_next_incomplete_daily_challenge_from_today()
        {
            let today_level = CurrentLevel::DailyChallenge { index };
            if today_level.level(&daily_challenges).is_left() {
                //Only change to this level if we have loaded it already

                return Some(today_level);
            } else {
                return Some(CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished));
            }
        }

        None
    }

    if let Some(level) = get_new_level(current_level, daily_challenge_completion, daily_challenges)
    {
        change_level_events.send(level.into());
    }
}

fn hide_splash() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        do_or_report_error(capacitor_bindings::splash_screen::SplashScreen::hide(
            0000.0,
        ));
    }
}

fn set_status_bar() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        use capacitor_bindings::status_bar::*;

        do_or_report_error(StatusBar::set_style(Style::Dark));
        #[cfg(feature = "android")]
        do_or_report_error(StatusBar::set_background_color("#2bb559"));
    }
}

fn watch_lifecycle(
    mut events: EventReader<AppLifeCycleEvent>,
    mut menu: ResMut<MenuState>,
    mut popup: ResMut<PopupState>,
) {
    for event in events.read() {
        match event {
            AppLifeCycleEvent::StateChange { .. } => {
                info!("State changed")
            }
            AppLifeCycleEvent::BackPressed => {
                if popup.0.is_some() {
                    popup.0 = None;
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
