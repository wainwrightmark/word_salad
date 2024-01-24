use bevy::{log, prelude::*};
//use capacitor_bindings::{app::AppInfo, device::*};
use crate::prelude::*;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use strum::EnumDiscriminants;

pub struct LogWatchPlugin;

impl Plugin for LogWatchPlugin {
    fn build(&self, app: &mut App) {
        //info!("Log watch plugin");
        app.add_systems(Update, watch_level_changes);
        app.add_systems(Update, watch_level_completion);
        app.add_systems(PostStartup, on_start);
    }
}

fn watch_level_changes(current_level: Res<CurrentLevel>, daily_challenges: Res<DailyChallenges>) {
    if current_level.is_changed() {
        //info!("Logging level changed");
        let level = current_level.level(&daily_challenges);
        let level = level
            .left()
            .map(|x| x.full_name())
            .unwrap_or_else(|| Ustr::from("Unknown"))
            .to_string();

        let event: LoggableEvent = LoggableEvent::StartLevel { level };
        event.try_log1();
    }
}

fn watch_level_completion(
    state: Res<FoundWordsState>,
    current_level: Res<CurrentLevel>,
    timer: Res<LevelTime>,
    daily_challenges: Res<DailyChallenges>,
) {
    if !state.is_changed() || state.is_added() {
        return;
    }
    if state.is_level_complete() {
        //info!("Logging level complete");
        let seconds = timer.total_elapsed().as_secs();

        let level = current_level.level(&daily_challenges);
        let level = level
            .left()
            .map(|x| x.full_name())
            .unwrap_or_else(|| Ustr::from("Unknown"))
            .to_string();

        let event = LoggableEvent::FinishLevel {
            level,
            seconds,
            hints_used: state.hints_used,
        };
        event.try_log1();
    }
}

pub fn on_start(mut pkv: ResMut<bevy_pkv::PkvStore>) {
    const KEY: &str = "UserExists";

    //info!("Logging on start");

    let user_exists = pkv.get::<bool>(KEY).ok().unwrap_or_default();

    if !user_exists {
        pkv.set(KEY, &true).unwrap();
    }

    spawn_and_run(log_start_async(user_exists));
}

async fn log_start_async<'a>(user_exists: bool) {
    {
        let device_id: DeviceIdentifier;
        #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
        {
            device_id = match capacitor_bindings::device::Device::get_id().await {
                Ok(device_id) => device_id.into(),
                Err(err) => {
                    crate::logging::try_log_error_message(format!("{err:?}"));
                    DeviceIdentifier::unknown()
                }
            };
        }

        #[cfg(not(any(feature = "android", feature = "ios", feature = "web")))]
        {
            #[cfg(feature = "steam")]
            {
                device_id = DeviceIdentifier::steam();
            }
            #[cfg(not(feature = "steam"))]
            {
                device_id = DeviceIdentifier::unknown();
            }
        }

        match DEVICE_ID.set(device_id.clone()) {
            Ok(()) => {
                debug!("Device id set {device_id:?}");
            }
            Err(err) => {
                error!("Error setting device id {err:?}")
            }
        }

        if !user_exists {
            let new_user: LoggableEvent;

            #[cfg(all(
                target_arch = "wasm32",
                any(feature = "android", feature = "ios", feature = "web")
            ))]
            {
                new_user = crate::wasm::new_user_async().await;
            }
            #[cfg(not(all(
                target_arch = "wasm32",
                any(feature = "android", feature = "ios", feature = "web")
            )))]
            {
                new_user = LoggableEvent::NewUser {
                    ref_param: None,
                    referrer: None,
                    gclid: None,
                    language: None,
                    device: None,
                    app: None,
                    platform: Platform::CURRENT,
                };
            }

            new_user.try_log_async1(device_id.clone()).await;
        }
        let application_start: LoggableEvent;
        #[cfg(all(
            target_arch = "wasm32",
            any(feature = "android", feature = "ios", feature = "web")
        ))]
        {
            application_start = crate::wasm::application_start().await;
        }

        #[cfg(not(all(
            target_arch = "wasm32",
            any(feature = "android", feature = "ios", feature = "web")
        )))]
        {
            application_start = LoggableEvent::ApplicationStart {
                ref_param: None,
                referrer: None,
                gclid: None,
            };
        }

        application_start.try_log_async1(device_id).await;
    }
}

#[must_use]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumDiscriminants, Event)]
#[serde(tag = "type")]
pub enum LoggableEvent {
    NewUser {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
        language: Option<String>,
        device: Option<DeviceInformation>,
        app: Option<LogAppInfo>,
        platform: Platform,
    },
    ApplicationStart {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
    },

    ClickShare,
    ShareOn {
        platform: String,
    },
    Warn {
        message: String,
    },
    Error {
        message: String,
    },

    Internal {
        message: String,
    },

    StartLevel {
        level: String,
    },

    FinishLevel {
        level: String,
        seconds: u64,
        hints_used: usize, //TODO permutation
    }, // GoAppStore {
    //     store: String,
    //     level: String,
    //     max_demo_level: u8,
    // },
    PermissionsRequested {
        given: String,
    },
    // FollowNewsLink,

    // NotificationClick,

    // ActedInTutorial
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::error::Error> for LoggableEvent {
    fn from(value: capacitor_bindings::error::Error) -> Self {
        Self::Error {
            message: value.to_string(),
        }
    }
}

// cSpell:ignore xaat

/// This token can only be used to ingest data into our bucket
const API_TOKEN: &str = "xaat-228cf6cb-9dd2-4023-a87e-87e2cbd7f853";

#[derive(Debug, Clone, Serialize)]
pub struct EventLog {
    pub device_id: DeviceIdentifier,
    #[serde(skip_serializing_if = "is_false")]
    pub resent: bool,
    pub event: LoggableEvent,
    #[serde(skip_serializing_if = "is_info_or_lower")]
    pub severity: Severity,
}

fn is_false(b: &bool) -> bool {
    !b
}

fn is_info_or_lower(severity: &Severity) -> bool {
    severity != &Severity::Warn && severity != &Severity::Error
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

// impl EventLog {
//     pub fn new_resent(device_id: DeviceIdentifier, event: LoggableEvent) -> Self {
//         let severity = event.get_severity();
//         Self {
//             device_id,
//             resent: true,
//             event,
//             severity,
//         }
//     }
// }

impl LogAppInfo {
    pub async fn try_get_async() -> Option<LogAppInfo> {
        #[cfg(any(feature = "android", feature = "ios"))]
        {
            capacitor_bindings::app::App::get_info()
                .await
                .ok()
                .map(|x| x.into())
            // crate::capacitor_bindings::get_or_log_error_async()
            //     .await
            //     .map(|x| x.into())
        }
        #[cfg(not(any(feature = "android", feature = "ios")))]
        {
            None
        }
    }
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
pub fn do_or_report_error(
    future: impl std::future::Future<Output = Result<(), capacitor_bindings::error::Error>> + 'static,
) {
    spawn_and_run(do_or_report_error_async(future))
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
pub async fn do_or_report_error_async(
    future: impl std::future::Future<Output = Result<(), capacitor_bindings::error::Error>>,
) {
    let result = future.await;

    match result {
        Ok(_) => {}
        Err(err) => {
            log::error!("{err:?}");
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }
}

pub fn try_log_error_message(message: String) {
    spawn_and_run(LoggableEvent::try_log_error_message_async2(message));
}

impl LoggableEvent {
    pub async fn try_log_error_message_async2(message: String) {
        const MESSAGES_TO_IGNORE: &[&str] = &[
            "Js Exception: Notifications not enabled on this device",
            "Js Exception: Notifications not supported in this browser.",
            "Js Exception: Player is not authenticated",
        ];

        if MESSAGES_TO_IGNORE.contains(&message.as_str()) {
            return;
        }

        Self::try_get_device_id_and_log_async(Self::Error { message }).await
    }

    pub async fn try_log_async1(self, device_id: DeviceIdentifier) {
        Self::try_log_async(self, device_id).await
    }

    /// Either logs the message or sends it to be retried later
    pub async fn try_log_async(data: impl Into<Self>, device_id: DeviceIdentifier) {
        //let user = Dispatch::<UserState>::new().get();
        let event = data.into();
        let severity = event.get_severity();

        let message = EventLog {
            event,
            device_id,
            resent: false,
            severity,
        };

        log::debug!("logged {message:?}");
        message.send_log_async().await;
    }

    pub async fn try_get_device_id_and_log_async(data: impl Into<Self>) {
        let device_id: DeviceIdentifier;
        #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
        {
            match capacitor_bindings::device::Device::get_id().await {
                Ok(id) => device_id = id.into(),
                Err(err) => {
                    log::error!("{err:?}");
                    return;
                }
            }
        }
        #[cfg(not(any(feature = "android", feature = "ios", feature = "web")))]
        {
            #[cfg(feature = "steam")]
            {
                device_id = DeviceIdentifier::steam();
            }
            #[cfg(not(feature = "steam"))]
            {
                device_id = DeviceIdentifier::unknown();
            }
        }

        Self::try_log_async(data, device_id).await
    }

    pub fn try_log1(self) {
        Self::try_log(self)
    }

    fn try_log(data: impl Into<Self> + 'static) {
        spawn_and_run(Self::try_get_device_id_and_log_async(data));
    }

    pub fn get_severity(&self) -> Severity {
        match self {
            LoggableEvent::Warn { .. } => Severity::Warn,
            LoggableEvent::Error { .. } => Severity::Error,
            _ => Severity::Info,
        }
    }
}

impl EventLog {
    pub async fn send_log_async(self) {
        Self::log_async(self).await
    }

    async fn try_log<T: Serialize>(data: &T) -> Result<(), reqwest::Error> {
        if !cfg!(debug_assertions) {
            //todo make this work properly on steam
            let client = reqwest::Client::new();
            let res = client
                .post("https://api.axiom.co/v1/datasets/word_salad/ingest")
                // .header("Authorization", format!("Bearer {API_TOKEN}"))
                .bearer_auth(API_TOKEN)
                .header("Content-Type", "application/json")
                .json(&[data])
                .send()
                .await?;

            res.error_for_status().map(|_| ())
        } else {
            Ok(())
        }
    }

    async fn log_async(data: Self) {
        let r = Self::try_log(&data).await;
        if let Err(err) = r {
            log::error!("Failed to log: {}", err);
            //Dispatch::<FailedLogsState>::new().apply(LogFailedMessage(data.event));
        } else {
            let discriminant: LoggableEvent = data.event;
            log::debug!("Log {discriminant:?} sent successfully",);
        }
    }
}
