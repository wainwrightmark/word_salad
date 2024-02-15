use bevy::prelude::*;
use capacitor_bindings::local_notifications::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;

#[allow(unused_imports)]
use crate::{logging, prelude::*};

#[cfg(any(feature = "ios", feature = "android"))]
const DAILY_CHALLENGE_CLICK_ACTION_ID: &str = "DailyChallengeClick";
const DAILY_CHALLENGE_ACTION_TYPE_ID: &str = "DailyChallenge";

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(any(feature = "ios", feature = "android"))]
        {
            app.add_systems(Startup, setup); //TODO setup at a different time
        }
    }
}

fn setup(writer: AsyncEventWriter<ChangeLevelEvent>, daily_challenges: Res<DailyChallenges>) {
    spawn_and_run(setup_notifications_async(writer, daily_challenges.clone()));
}

async fn setup_notifications_async(
    writer: AsyncEventWriter<ChangeLevelEvent>,
    daily_challenges: DailyChallenges,
) {
    let schedule_options = LocalNotificationSchema::builder()
        .title("Work Salad")
        .body("Today's Daily Challenge is ready")
        .summary_text("Today's Daily Challenge is ready")
        .id(-1225158782) //Very Random number
        .action_type_id(DAILY_CHALLENGE_ACTION_TYPE_ID)
        .small_icon("notification_icon")
        .large_icon("notification_icon")
        .icon_color("#86AEEA")
        .schedule(ScheduleOn::builder().hour(6).build())
        .auto_cancel(true)
        .build();

    let on_action = move |action: ActionPerformed| {
        if action.action_id == DAILY_CHALLENGE_ACTION_TYPE_ID || action.action_id == "tap" {
            bevy::log::info!("Clicked Action");

            let new_level = CurrentLevel::DailyChallenge {
                index: DailyChallenges::get_today_index(),
            };

            let level_to_send = if new_level.level(&daily_challenges).is_left() {
                //Only change to this level if we have loaded it already

                new_level
            } else {
                CurrentLevel::NonLevel(NonLevel::DailyChallengeFinished)
            };

            writer
                .send_blocking(level_to_send.into())
                .expect("Channel closed prematurely");
        }
    };

    #[cfg(any(feature = "ios", feature = "android"))]
    {
        match LocalNotifications::check_permissions().await {
            Ok(permissions) => match permissions.display {
                PermissionState::Prompt | PermissionState::PromptWithRationale => {
                    match LocalNotifications::request_permissions().await {
                        Ok(new_permission_status) => {
                            let given = match new_permission_status.display {
                                PermissionState::Prompt => "Prompt",
                                PermissionState::PromptWithRationale => "PromptWithRationale",
                                PermissionState::Granted => "Granted",
                                PermissionState::Denied => "Denied",
                            }
                            .to_string();
                            let event = LoggableEvent::PermissionsRequested { given };

                            logging::LoggableEvent::try_get_device_id_and_log_async(event).await;

                            if new_permission_status.display == PermissionState::Denied {
                                return;
                            }
                        }
                        Err(err) => {
                            let event: LoggableEvent = err.into();
                            logging::LoggableEvent::try_get_device_id_and_log_async(event).await;
                            return;
                        }
                    }
                }
                PermissionState::Granted => {}
                PermissionState::Denied => {
                    return;
                }
            },
            Err(err) => {
                let event: LoggableEvent = err.into();
                logging::LoggableEvent::try_get_device_id_and_log_async(event).await;
                return;
            }
        }

        bevy::log::debug!("Registering Action Types");
        crate::logging::do_or_report_error_async({
            let action_type_options = RegisterActionTypesOptions {
                types: vec![ActionType {
                    id: DAILY_CHALLENGE_ACTION_TYPE_ID.to_string(),
                    actions: vec![Action {
                        id: DAILY_CHALLENGE_CLICK_ACTION_ID.to_string(),
                        title: "Play Now".to_string(),
                    }],
                }],
            };
            LocalNotifications::register_action_types(action_type_options)
        })
        .await;
    }

    schedule_notification(schedule_options, on_action).await;
}

async fn schedule_notification<F: Fn(ActionPerformed) + 'static>(
    schedule_options: impl Into<ScheduleOptions>,
    on_action: F,
) {
    bevy::log::debug!("Scheduling local notification...");
    let schedule_result = LocalNotifications::schedule(schedule_options).await;

    match schedule_result {
        Ok(sr) => {
            bevy::log::debug!("Notification Scheduled {:?}", sr.notifications);
        }
        Err(err) => {
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }

    bevy::log::debug!("Registering Action Listener");
    let listener_result = LocalNotifications::add_action_performed_listener(on_action).await;
    match listener_result {
        Ok(lr) => {
            lr.leak();
        }
        Err(err) => {
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }
    bevy::log::debug!("Action Listener Registered");
}
