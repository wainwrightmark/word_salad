use crate::{asynchronous, prelude::*};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use capacitor_bindings::{device::Device, share::ShareOptions};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::UrlSearchParams;

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, resizer);
    }
}

#[derive(Default)]
struct LastSize {
    pub width: f32,
    pub height: f32,
}

fn resizer(
    //TODO move to nice bevy utils
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut window_resized_events: EventWriter<bevy::window::WindowResized>,
    mut last_size: Local<LastSize>,
) {
    let window = web_sys::window().expect("no global `window` exists");
    let mut width: f32 = window.inner_width().unwrap().as_f64().unwrap() as f32;
    let mut height: f32 = window.inner_height().unwrap().as_f64().unwrap() as f32;
    if width != last_size.width || height != last_size.height {
        if let Ok((window_entity, mut window)) = windows.get_single_mut() {
            *last_size = LastSize { width, height };

            let constraints = window.resize_constraints;

            width = width.clamp(constraints.min_width, constraints.max_width);
            height = height.clamp(constraints.min_height, constraints.max_height);

            let p_width = width * window.scale_factor() as f32;
            let p_height = height * window.scale_factor() as f32;
            window
                .resolution
                .set_physical_resolution(p_width.floor() as u32, p_height.floor() as u32);
            window_resized_events.send(bevy::window::WindowResized {
                window: window_entity,
                height,
                width,
            });

            debug!(
                "Resizing to {:?},{:?} with scale factor of {}",
                width,
                height,
                window.scale_factor()
            );
        }
    }
}

pub fn share(data: String) {
    asynchronous::spawn_and_run(async {
        share_game_async(data).await;
    });
}

async fn share_game_async(data: String) {
    let device_id = capacitor_bindings::device::Device::get_id()
        .await
        .unwrap_or_else(|_| capacitor_bindings::device::DeviceId {
            identifier: "unknown".to_string(),
        });

    LoggableEvent::ClickShare
        .try_log_async1(device_id.clone().into())
        .await;

    let result = capacitor_bindings::share::Share::share(
        ShareOptions::builder()
            .title("Word Salad")
            .text(data)
            //.url("https://wordsalad.online/")
            .build(),
    )
    .await;

    match result {
        Ok(share_result) => {
            if let Some(platform) = share_result.activity_type {
                LoggableEvent::ShareOn { platform }
                    .try_log_async1(device_id.into())
                    .await;
            }

            bevy::log::info!("Share succeeded")
        }
        Err(_) => info!("Share failed"),
    }
}

pub fn get_daily_from_location() -> Option<usize> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    try_daily_index_from_path(path)
}

fn try_daily_index_from_path(path: String) -> Option<usize> {
    //info!("{path}");
    if path.is_empty() || path.eq_ignore_ascii_case("/") {
        return None;
    }

    if path.to_ascii_lowercase().starts_with("/daily/") {
        //info!("{path} starts with daily");
        let data = path[7..].to_string();

        let index = usize::from_str_radix(data.trim(), 10)
            .ok()?
            .checked_sub(1)?;

        let today_index = DailyChallenges::get_today_index();

        if index <= today_index {
            //info!("{path} index is legit");
            return Some(index);
        }
    }
    return None;
}

pub fn get_game_from_location() -> Option<DesignedLevel> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    designed_level_try_from_path(path)
}

fn designed_level_try_from_path(path: String) -> Option<DesignedLevel> {
    //info!("path: {path}");

    if path.is_empty() || path.eq_ignore_ascii_case("/") {
        return None;
    }

    if path.to_ascii_lowercase().starts_with("/game/") {
        //log::info!("Path starts with game");
        let data = path[6..].to_string();
        //log::info!("{data}");

        use base64::Engine;

        let data = base64::engine::general_purpose::URL_SAFE
            .decode(data)
            .ok()?;

        let data = String::from_utf8(data).ok()?;

        match DesignedLevel::from_tsv_line(&data.trim()) {
            Ok(data) => Some(data),
            Err(err) => {
                error!("{err}");
                None
            }
        }
    } else {
        None
    }
}

pub fn open_link(url: &str) {
    use web_sys::window;

    let window = match window() {
        Some(window) => window,
        None => {
            error!("Could not get window to open link");
            return;
        }
    };

    match window.open_with_url_and_target(url, "_top") {
        Ok(_) => {}
        Err(err) => {
            error!("{err:?}")
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen()]
extern "C" {
    #[wasm_bindgen(catch, final, js_namespace = ["Capacitor", "Plugins", "ScreenRecorder"], js_name="start" )]
    async fn start_screen_record_extern() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, final, js_namespace = ["Capacitor", "Plugins", "ScreenRecorder"], js_name="stop" )]
    async fn stop_screen_record_extern() -> Result<(), JsValue>;

    #[wasm_bindgen(catch, final, js_namespace = ["Capacitor", "Plugins", "ScreenRecorder"], js_name="recording_state" )]
    async fn screen_recording_state_extern() -> Result<JsValue, JsValue>;
}

pub async fn stop_screen_record()-> Result<(), capacitor_bindings::error::Error>{
    capacitor_bindings::helpers::run_unit_unit(stop_screen_record_extern).await
}

pub async fn start_screen_record()->Result<VideoRecordingState, capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_value(start_screen_record_extern).await
}


pub async fn screen_recording_state()->Result<VideoRecordingState, capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_value(screen_recording_state_extern).await
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoRecordingState {
    pub value: VideoRecordingStateEnum,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoRecordingStateEnum {
    Idle,
    Paused,
    Recording,
    StartDelay,
    Error,
    Unknown,
}

#[wasm_bindgen::prelude::wasm_bindgen(module = "/video.js")]
extern "C" {
    #[wasm_bindgen(catch, final, js_name = "startVideo")]
    async fn start_video_extern() -> Result<(), JsValue>;

    #[wasm_bindgen(catch, final, js_name = "stopVideo")]
    async fn stop_video_extern() -> Result<(), JsValue>;
}

pub async fn start_selfie_mode_video()->Result<(), capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_unit(start_video_extern).await
}


pub async fn stop_selfie_mode_video()->Result<(), capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_unit(stop_video_extern).await
}


pub async fn application_start() -> LoggableEvent {
    let search_params = get_url_search_params().await;

    let ref_param = search_params.clone().and_then(|x| x.get("ref"));
    let gclid = search_params.and_then(|x| x.get("gclid"));
    let referrer = get_referrer();

    //info!("{:?}",event);
    LoggableEvent::ApplicationStart {
        ref_param,
        referrer,
        gclid,
    }
}

pub async fn new_user_async() -> LoggableEvent {
    let search_params = get_url_search_params().await;

    let ref_param = search_params.clone().and_then(|x| x.get("ref"));
    let gclid = search_params.and_then(|x| x.get("gclid"));
    let referrer = get_referrer();

    let language = Device::get_language_tag().await.map(|x| x.value).ok();
    let device = Device::get_info().await.map(|x| x.into()).ok();

    let app = LogAppInfo::try_get_async().await;

    LoggableEvent::NewUser {
        ref_param,
        referrer,
        gclid,
        language,
        device,
        app,
        platform: Platform::CURRENT,
    }
}

fn get_referrer() -> Option<String> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let referrer = document.referrer();
    if referrer.is_empty() {
        return None;
    }
    Some(referrer)
}

async fn get_url_search_params() -> Option<UrlSearchParams> {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        let url = capacitor_bindings::app::App::get_launch_url()
            .await
            .ok()??;

        let url = web_sys::Url::new(&url.url).ok()?;
        let params = url.search_params();
        return Some(params);
    }

    #[cfg(not(any(feature = "android", feature = "ios")))]
    {
        use web_sys::window;
        let window = window()?;
        let search = window.location().search().ok()?;
        let params = UrlSearchParams::new_with_str(search.as_str()).ok()?;
        Some(params)
    }
}

/// An exception thrown by a javascript function
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsException {
    pub message: String,
}

impl TryFrom<JsValue> for JsException {
    type Error = ();

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        if let Ok(exception) = serde_wasm_bindgen::from_value::<JsException>(value.clone()) {
            Ok(JsException {
                message: exception.message,
            })
        } else {
            Err(())
        }
    }
}
