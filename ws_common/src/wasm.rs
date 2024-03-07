use crate::{asynchronous, prelude::*};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use capacitor_bindings::{device::Device, share::ShareOptions};
use nice_bevy_utils::async_event_writer;
use nice_bevy_utils::CanRegisterAsyncEvent;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::{closure::Closure, JsValue};
use web_sys::UrlSearchParams;

pub struct WasmPlugin;

impl Plugin for WasmPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            resizer.run_if(|e: EventReader<WebWindowResizedEvent>| !e.is_empty()),
        );
        app.add_systems(Startup, register_on_window_resized);
        app.add_systems(PostStartup, update_insets);

        app.register_async_event::<WebWindowResizedEvent>();
    }
}

#[derive(Default, PartialEq, Clone, Copy, Debug, Event)]
struct WebWindowResizedEvent;

fn register_on_window_resized(writer: async_event_writer::AsyncEventWriter<WebWindowResizedEvent>) {
    let web_window = web_sys::window().expect("no global `window` exists");

    let closure = Closure::<dyn Fn()>::new(move || {
        writer.send_or_panic(WebWindowResizedEvent);
    });

    web_window.set_onresize(Some(closure.as_ref().unchecked_ref()));

    std::mem::forget(closure);
}

fn resizer(
    mut events: EventReader<WebWindowResizedEvent>,
    //TODO move to nice bevy utils
    mut windows: Query<(Entity, &mut Window), With<PrimaryWindow>>,
    mut window_resized_events: EventWriter<bevy::window::WindowResized>,
) {
    for (index, _) in events.read().enumerate() {
        if index > 0 {
            continue;
        }

        if let Ok((window_entity, mut window)) = windows.get_single_mut() {
            let mut current_size = WindowSizeValues::from_web_window();
            current_size.clamp_to_resize_constraints(&window.resize_constraints);

            window.resolution = current_size.to_window_resolution();

            window_resized_events.send(bevy::window::WindowResized {
                window: window_entity,
                height: current_size.height,
                width: current_size.width,
            });

            info!("Resizing to {current_size:?}");
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

    try_daily_index_from_path(path.as_str())
}

pub fn get_game_from_location() -> Option<DesignedLevel> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    DesignedLevel::try_from_path(path.as_ref())
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

pub async fn stop_screen_record() -> Result<(), capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_unit(stop_screen_record_extern).await
}

pub async fn start_screen_record() -> Result<VideoRecordingState, capacitor_bindings::error::Error>
{
    capacitor_bindings::helpers::run_unit_value(start_screen_record_extern).await
}

pub async fn screen_recording_state(
) -> Result<VideoRecordingState, capacitor_bindings::error::Error> {
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

pub async fn start_selfie_mode_video() -> Result<(), capacitor_bindings::error::Error> {
    capacitor_bindings::helpers::run_unit_unit(start_video_extern).await
}

pub async fn stop_selfie_mode_video() -> Result<(), capacitor_bindings::error::Error> {
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

fn update_insets(mut insets: ResMut<InsetsResource>) {
    if let Some(new_insets) = get_insets() {
        debug!("{:?}", new_insets.clone());
        insets.0 = new_insets;
    }
}

fn get_insets() -> Option<Insets> {
    let window = web_sys::window()?;
    let document = window.document()?.document_element()?;
    let style = window.get_computed_style(&document).ok()??;

    let top = style
        .get_property_value("--sat")
        .ok()
        .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
        .unwrap_or_default();
    // let left = style
    //     .get_property_value("--sal")
    //     .ok()
    //     .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
    //     .unwrap_or_default();
    // let right = style
    //     .get_property_value("--sar")
    //     .ok()
    //     .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
    //     .unwrap_or_default();
    // let bottom = style
    //     .get_property_value("--sab")
    //     .ok()
    //     .and_then(|x| x.trim_end_matches("px").parse::<f32>().ok())
    //     .unwrap_or_default();

    let insets = Insets::new(top);
    Some(insets)
}
