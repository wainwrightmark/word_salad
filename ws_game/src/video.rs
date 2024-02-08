use bevy::prelude::*;
use maveric::helpers::MavericContext;
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};

use ws_core::layout::entities::SelfieMode;

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VideoResource::default());

        app.register_async_event::<VideoEvent>();
        app.add_systems(Update, handle_video_event);
        // app.add_systems(
        //     Update,
        //     check_recording.run_if(|v: Res<VideoResource>| v.is_recording),
        // );
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
    pub is_recording: bool,
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
}

// #[allow(unused_variables)]
// #[allow(dead_code)]
fn handle_video_event(mut res: ResMut<VideoResource>, mut events: EventReader<VideoEvent>) {
    for ev in events.read() {
        match ev {
            VideoEvent::SelfieModeStarted => {
                res.is_selfie_mode = true;
            }
            VideoEvent::SelfieModeStopped => res.is_selfie_mode = false,
            VideoEvent::RecordingStarted => {
                res.is_recording = true;
            }
            VideoEvent::RecordingStopped => {
                res.is_recording = false;
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
                        writer
                            .send_async(VideoEvent::RecordingStarted)
                            .await
                            .unwrap();
                    }

                    _ => {
                        crate::platform_specific::show_toast_async("Failed to start Screen Record")
                            .await;
                        writer
                            .send_async(VideoEvent::RecordingStopped)
                            .await
                            .unwrap();
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
                writer
                    .send_async(VideoEvent::RecordingStopped)
                    .await
                    .unwrap();
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
            Ok(()) => writer
                .send_async(VideoEvent::SelfieModeStarted)
                .await
                .unwrap(),
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
            Ok(()) => writer
                .send_async(VideoEvent::SelfieModeStopped)
                .await
                .unwrap(),
            Err(err) => {
                crate::platform_specific::show_toast_async("Could not stop Selfie Mode").await;
                error!("{}", err)
            }
        }
    }
}

pub const ALLOW_VIDEO: bool = {
    if cfg!(target_arch = "wasm32") {
        true
    } else {
        false
    }
};
