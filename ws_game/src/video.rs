use bevy::prelude::*;
use maveric::{helpers::MavericContext, plugin::CanRegisterMaveric};
use nice_bevy_utils::{
    async_event_writer::AsyncEventWriter, CanInitTrackedResource, CanRegisterAsyncEvent,
    TrackableResource,
};
use serde::{Deserialize, Serialize};

use crate::prelude::selfie_popup::SelfiePopup;

//use crate::{asynchronous, wasm::JsException};

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VideoResource::default());
        app.init_tracked_resource::<SelfieModeHistory>();

        app.register_maveric::<SelfiePopup>();
        app.register_async_event::<VideoEvent>();
        #[cfg(target_arch = "wasm32")]
        {
            app.add_systems(Update, handle_video_event);
        }
    }
}

#[derive(Debug, Event, Clone, Copy, PartialEq, Eq)]
pub enum VideoEvent {
    VideoStarted,
    VideoStopped,
}

#[derive(Default, Resource, MavericContext)]
pub struct VideoResource {
    pub is_selfie_mode: bool,
}

#[derive(Default, Resource, Clone, PartialEq, Serialize, Deserialize, MavericContext)]
pub struct SelfieModeHistory {
    pub has_entered_selfie_mode: bool,
}

impl TrackableResource for SelfieModeHistory {
    const KEY: &'static str = "SelfieModeHistory";
}

#[allow(unused_variables)]
#[allow(dead_code)]
fn handle_video_event(
    mut res: ResMut<VideoResource>,
    mut history: ResMut<SelfieModeHistory>,
    mut events: EventReader<VideoEvent>,
) {
    for ev in events.read() {
        match ev {
            VideoEvent::VideoStarted => {
                res.is_selfie_mode = true;

                if !history.has_entered_selfie_mode {
                    history.has_entered_selfie_mode = true;
                }
            }
            VideoEvent::VideoStopped => res.is_selfie_mode = false,
        }
    }
}

impl VideoResource {
    #[allow(unused_variables)]
    pub fn toggle_video_streaming(&self, writer: AsyncEventWriter<VideoEvent>) {
        #[cfg(target_arch = "wasm32")]
        {
            if self.is_selfie_mode {
                crate::wasm::stop_video();
                writer.send_blocking(VideoEvent::VideoStopped).unwrap();
            } else {
                crate::asynchronous::spawn_and_run(start_video_async(writer));
            }
        }
    }
}

#[allow(unused_variables)]
#[allow(dead_code)]
async fn start_video_async(writer: AsyncEventWriter<VideoEvent>) {
    #[cfg(target_arch = "wasm32")]
    {
        let result = crate::wasm::start_video().await;

        match result {
            Ok(()) => writer.send_async(VideoEvent::VideoStarted).await.unwrap(),
            Err(err) => match crate::wasm::JsException::try_from(err) {
                Ok(e) => error!("{}", e.message),
                Err(()) => error!("Error Starting Video"),
            },
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
