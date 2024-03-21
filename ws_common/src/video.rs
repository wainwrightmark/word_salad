use bevy::prelude::*;
use chrono::{DateTime, Utc};
use maveric::helpers::MavericContext;
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};

use ws_core::layout::entities::SelfieMode;

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VideoResource::default());

        app.register_async_event::<VideoEvent>();
        app.add_systems(Update, handle_video_event);
    }
}

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq)]
pub enum VideoEvent {
    SelfieModeStarted,
    SelfieModeStopped,

    RecordingStarted,
    RecordingStopped,
}

#[derive(Debug, Default, Resource, MavericContext)]
pub struct VideoResource {
    pub is_selfie_mode: bool,
    pub recording_since: Option<DateTime<Utc>>,
}

impl VideoResource {
    pub fn selfie_mode(&self) -> SelfieMode {
        SelfieMode {
            is_selfie_mode: self.is_selfie_mode,
        }
    }

    pub fn show_recording_button(&self) -> bool {
        cfg!(any(feature = "android", feature = "ios")) && self.is_selfie_mode
    }

    pub fn is_recording(&self) -> bool {
        self.recording_since.is_some()
    }
}

// #[allow(unused_variables)]
// #[allow(dead_code)]
fn handle_video_event(mut res: ResMut<VideoResource>, mut events: EventReader<VideoEvent>) {
    for ev in events.read() {
        match ev {
            VideoEvent::SelfieModeStarted => {
                crate::logging::LoggableEvent::SelfieModeStarted.try_log1();
                res.is_selfie_mode = true;
            }
            VideoEvent::SelfieModeStopped => {
                if res.is_selfie_mode {
                    res.is_selfie_mode = false;
                    crate::logging::LoggableEvent::SelfieModeStopped.try_log1();
                }
            }
            VideoEvent::RecordingStarted => {
                res.recording_since = Some(chrono::Utc::now());
            }
            VideoEvent::RecordingStopped => {
                if let Some(since) = res.recording_since {
                    res.recording_since = None;
                    let elapsed = chrono::Utc::now().signed_duration_since(since);
                    let recording_seconds = elapsed.num_seconds();

                    crate::logging::LoggableEvent::RecordingStopped { recording_seconds }.try_log1()
                }
            }
        }
        crate::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

impl VideoResource {
    #[allow(unused_variables)]
    pub fn toggle_selfie_mode(&self, writer: AsyncEventWriter<VideoEvent>) {
        #[cfg(target_arch = "wasm32")]
        {
            if self.is_selfie_mode {
                crate::asynchronous::spawn_and_run(stop_selfie_mode_async(writer));
            } else {
                crate::asynchronous::spawn_and_run(start_selfie_mode_async(writer));
            }
        }
    }
}

#[allow(unused_variables)]
pub async fn start_screen_record(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        let r = crate::wasm::start_screen_record().await;
        crate::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // info!("Starting screen record");
        match r {
            Ok(r) => {
                //info!("Recording started: state {r:?}");
                match r.value {
                    crate::wasm::VideoRecordingStateEnum::Recording => {
                        writer.send_or_panic(VideoEvent::RecordingStarted);
                        crate::logging::LoggableEvent::RecordingStarted
                            .try_log_async2()
                            .await
                    }

                    _ => {
                        crate::platform_specific::show_toast_async("Failed to start Screen Record")
                            .await;
                        writer.send_or_panic(VideoEvent::RecordingStopped);
                        crate::logging::LoggableEvent::RecordingNotStarted
                            .try_log_async2()
                            .await
                    }
                }
            }
            Err(err) => {
                crate::platform_specific::show_toast_async("Could not start Screen Record").await;
                error!("{}", err)
            }
        }
    }
}

#[allow(unused_variables)]
pub async fn stop_screen_record(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
    {
        let r = crate::wasm::stop_screen_record().await;
        crate::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match r {
            Ok(()) => {
                writer.send_or_panic(VideoEvent::RecordingStopped);
            }
            Err(err) => {
                crate::platform_specific::show_toast_async("Could not stop Screen Record").await;
                error!("{}", err)
            }
        }
    }
}

#[allow(unused_variables)]
#[allow(dead_code)]
async fn start_selfie_mode_async(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        let result = crate::wasm::start_selfie_mode_video().await;
        crate::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match result {
            Ok(()) => writer.send_or_panic(VideoEvent::SelfieModeStarted),
            Err(err) => {
                crate::platform_specific::show_toast_async("Could not start Selfie Mode").await;
                error!("{}", err)
            }
        }
    }
}

#[allow(unused_variables)]
#[allow(dead_code)]
async fn stop_selfie_mode_async(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        let result = crate::wasm::stop_selfie_mode_video().await;
        crate::startup::ADDITIONAL_TRACKING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match result {
            Ok(()) => writer.send_or_panic(VideoEvent::SelfieModeStopped),
            Err(err) => {
                crate::platform_specific::show_toast_async("Could not stop Selfie Mode").await;
                error!("{}", err)
            }
        }
    }
}

pub const ALLOW_VIDEO: bool = { cfg!(target_arch = "wasm32") };
