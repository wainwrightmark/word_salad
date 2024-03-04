use chrono::{DateTime, Utc};
use nice_bevy_utils::{async_event_writer, CanRegisterAsyncEvent};
use ws_common::prelude::*;

pub struct AppLifecyclePlugin;
impl Plugin for AppLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, hide_splash);

        app.register_async_event::<AppLifeCycleEvent>();
        app.add_systems(Startup, begin_lifecycle);
        app.add_systems(
            Update,
            watch_lifecycle.run_if(|ev: EventReader<AppLifeCycleEvent>| !ev.is_empty()),
        );
        app.add_systems(Startup, set_status_bar.after(hide_splash));
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Event, PartialEq)]
enum AppLifeCycleEvent {
    StateChange {
        is_active: bool,
        time_sent: DateTime<Utc>,
    },
    BackPressed,
    UrlOpened {
        url: String,
    },
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
    mut level_time: ResMut<LevelTime>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    for event in events.read() {
        match event {
            AppLifeCycleEvent::StateChange {
                is_active,
                time_sent,
            } => {
                if *is_active {
                    info!("State changed to active {time_sent}")
                } else {
                    info!("State changed to inactive {time_sent}");

                    match level_time.as_ref() {
                        LevelTime::Running { since, additional } => {
                            let additional = chrono::Duration::from_std(*additional)
                                .unwrap_or(chrono::Duration::zero())
                                + time_sent.signed_duration_since(since);

                            let additional = additional.to_std().unwrap_or_default();
                            *level_time = LevelTime::Running {
                                since: chrono::Utc::now(),
                                additional,
                            };
                        }
                        LevelTime::Paused { .. } => {}
                        LevelTime::Finished { .. } => {}
                    }
                }
            }
            AppLifeCycleEvent::BackPressed => {
                if popup.0.is_some() {
                    popup.0 = None;
                } else if !menu.is_closed() {
                    menu.close();
                }
            }
            AppLifeCycleEvent::UrlOpened { url } => {
                info!("Url opened event '{url}'");
                if let Some(daily_index) = try_daily_index_from_path(&url) {
                    let new_level = CurrentLevel::DailyChallenge { index: daily_index };
                    change_level_events.send(new_level.into());
                } else if let Some(level) = DesignedLevel::try_from_path(&url) {
                    let custom_level = CurrentLevel::Custom {
                        name: level.full_name().clone(),
                    };
                    set_custom_level(level);

                    change_level_events.send(custom_level.into());
                }
            }
        }
    }
}

fn begin_lifecycle(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    spawn_and_run(disable_back_async(writer.clone()));
    spawn_and_run(on_resume(writer.clone()));
    spawn_and_run(handle_app_url_open(writer));
}

async fn handle_app_url_open<'a>(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        //info!("Registering app url open");
        let result = capacitor_bindings::app::App::add_app_url_open_listener(
            move |x: capacitor_bindings::app::URLOpenListenerEvent| {
                let event = AppLifeCycleEvent::UrlOpened { url: x.url };
                ws_common::startup::ADDITIONAL_TRACKING
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                writer.send_or_panic(event);
            },
        )
        .await;

        match result {
            Ok(handle) => {
                //info!("Leading back button");
                handle.leak();
            }
            Err(err) => {
                ws_common::logging::try_log_error_message(format!(
                    "Error Registering app url open '{err}'"
                ));
            }
        }
    }
}

async fn disable_back_async<'a>(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(feature = "android")]
    {
        info!("Disabling back");
        let result = capacitor_bindings::app::App::add_back_button_listener(move |_| {
            ws_common::startup::ADDITIONAL_TRACKING
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            writer.send_or_panic(AppLifeCycleEvent::BackPressed);
        })
        .await;

        match result {
            Ok(handle) => {
                //info!("Leading back button");
                handle.leak();
            }
            Err(err) => {
                ws_common::logging::try_log_error_message(format!("{err}"));
            }
        }
    }
}

async fn on_resume(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        //info!("Setting on_resume");
        let result = capacitor_bindings::app::App::add_state_change_listener(move |x| {
            let event = AppLifeCycleEvent::StateChange {
                is_active: x.is_active,
                time_sent: chrono::Utc::now(),
            };
            ws_common::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            writer.send_or_panic(event);
        })
        .await;

        match result {
            Ok(handle) => {
                handle.leak();
            }
            Err(err) => {
                ws_common::logging::try_log_error_message(format!("{err}"));
            }
        }
    }
}
