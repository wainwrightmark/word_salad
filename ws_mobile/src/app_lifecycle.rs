use nice_bevy_utils::{async_event_writer, CanRegisterAsyncEvent};
use ws_common::prelude::*;

pub struct AppLifecyclePlugin;
impl Plugin for AppLifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, hide_splash);

        app.register_async_event::<AppLifeCycleEvent>();
        app.add_systems(Startup, begin_lifecycle);
        app.add_systems(Update, watch_lifecycle);
        app.add_systems(Startup, set_status_bar.after(hide_splash));
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Event, PartialEq)]
enum AppLifeCycleEvent {
    StateChange { is_active: bool },
    BackPressed,
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

async fn disable_back_async<'a>(writer: async_event_writer::AsyncEventWriter<AppLifeCycleEvent>) {
    #[cfg(feature = "android")]
    {
        info!("Disabling back");
        let result = capacitor_bindings::app::App::add_back_button_listener(move |_| {
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
            };
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
