use std::sync::atomic::AtomicUsize;

pub use crate::prelude::*;
use crate::{achievements::AchievementsPlugin, input::InputPlugin, motion_blur::MotionBlurPlugin};
use bevy::{asset::embedded_asset, log::LogPlugin, window::RequestRedraw};
use nice_bevy_utils::window_size::WindowSizePlugin;
use ws_core::layout::entities::*;

pub fn setup_app(extra_setup: impl FnOnce(&mut App)) {
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
        .add_plugins(DefaultPlugins.set(window_plugin).set(log_plugin).build())
        .add_systems(Startup, setup_system);
    app.add_plugins(EmbeddedAssetPlugin);

    app.add_plugins(ClearColorPlugin);

    app.add_plugins(WordlinePlugin);
    app.register_maveric::<ViewRoot>();
    app.register_maveric::<RecordingButtonRoot>();
    app.register_maveric::<WordSaladLogoRoot>();
    app.register_maveric::<MenuRoot>();
    app.add_plugins(HintsRemainingPlugin);
    app.add_plugins(StatePlugin);
    app.add_plugins(LevelTimePlugin);
    app.add_plugins(ShapesPlugin);
    app.add_plugins(PopupPlugin);
    app.add_plugins(LogWatchPlugin);
    app.add_plugins(DailyChallengePlugin);
    app.add_plugins(StreakPlugin);
    app.add_plugins(MotionBlurPlugin);
    app.add_plugins(WordsPlugin);
    app.add_plugins(MonetizationPlugin);
    app.add_plugins(AchievementsPlugin);

    app.register_transition::<TransformRotationYLens>();
    app.register_transition::<TransformTranslationLens>();
    app.register_transition::<TransformScaleLens>();
    app.register_transition::<(TransformTranslationLens, TransformScaleLens)>();

    app.add_plugins(InputPlugin);

    app.add_plugins(WindowSizePlugin::<SaladWindowBreakPoints>::default());

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(crate::wasm::WasmPlugin);
    app.add_plugins(VideoPlugin);

    app.add_plugins(AdsPlugin);

    app.add_systems(PostStartup, choose_level_on_game_load);

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.insert_resource(bevy_pkv::PkvStore::new_in_dir("saves"));
    }
    #[cfg(target_arch = "wasm32")]
    {
        app.insert_resource(bevy_pkv::PkvStore::new("bleppo", "word_salad"));
    }

    app.insert_resource(bevy::winit::WinitSettings {
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

    extra_setup(&mut app);

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
                        return Some(CurrentLevel::NonLevel(NonLevel::DailyChallengeNotLoaded {
                            goto_level: daily_index,
                        }));
                    }
                } else {
                    return None;
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
                } else {
                    return None;
                }
            }
        }

        match current_level.as_ref() {
            CurrentLevel::Tutorial { .. } | CurrentLevel::NonLevel(NonLevel::BeforeTutorial) => {
                return None;
            }
            _ => {}
        }

        let nl: CurrentLevel = daily_challenge_completion
            .get_next_incomplete_daily_challenge_from_today(&daily_challenges)
            .into();

        Some(nl)
    }

    if let Some(level) = get_new_level(current_level, daily_challenge_completion, daily_challenges)
    {
        change_level_events.send(level.into());
    }
}

pub struct EmbeddedAssetPlugin;

impl Plugin for EmbeddedAssetPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "", "../../assets/fonts/Montserrat-Bold.ttf");
        embedded_asset!(app, "", "../../assets/fonts/Montserrat-Regular.ttf");
        embedded_asset!(app, "", "../../assets/fonts/Montserrat-SemiBold.ttf");
        embedded_asset!(app, "", "../../assets/fonts/ws_icons.ttf");

        embedded_asset!(app, "", "../../assets/images/logo1024.png");
        embedded_asset!(app, "", "../../assets/images/steks_button.png");

        embedded_asset!(app, "", "../../assets/shaders/fill/fill_with_outline.wgsl");
        embedded_asset!(app, "", "../../assets/shaders/fill/fireworks.wgsl");
        embedded_asset!(app, "", "../../assets/shaders/fill/gradient.wgsl");
        embedded_asset!(
            app,
            "",
            "../../assets/shaders/fill/horizontal_gradient.wgsl"
        );
        embedded_asset!(app, "", "../../assets/shaders/fill/simple.wgsl");
        embedded_asset!(app, "", "../../assets/shaders/fill/sparkle.wgsl");

        embedded_asset!(app, "", "../../assets/shaders/sdf/box.wgsl");
        embedded_asset!(app, "", "../../assets/shaders/sdf/circle.wgsl");
        embedded_asset!(app, "", "../../assets/shaders/sdf/word_line_segment.wgsl");
    }
}