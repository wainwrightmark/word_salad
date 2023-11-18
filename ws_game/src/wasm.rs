use crate::{asynchronous, prelude::*};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::ShareData;

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

pub fn share() {


    asynchronous::spawn_and_run(async {
        let Some(window) = web_sys::window() else {
            return;
        };
        let navigator = window.navigator();
        let mut share_data = ShareData::new();
        share_data.title("Word Salad");
        share_data.url("wordsalad.online"); //TODO pipe in time
        let result = wasm_bindgen_futures::JsFuture::from(navigator.share_with_data(&share_data)).await;
        match result {
            Ok(_) => {

            },
            Err(err) => {
                match JsException::try_from(err){
                    Ok(e) => error!("{}", e.message),
                    Err(_) => error!("Error whilst sharing"),
                }

            },
        }
    });
}

pub fn get_game_from_location() -> Option<DesignedLevel> {
    let window = web_sys::window()?;
    let location = window.location();
    let path = location.pathname().ok()?;

    DesignedLevel::try_from_path(path)
}

#[wasm_bindgen::prelude::wasm_bindgen(module = "/video.js")]
extern "C" {
    #[wasm_bindgen(catch, final, js_name = "startVideo")]
    pub async fn start_video() -> Result<(), JsValue>;

    #[wasm_bindgen(final, js_name = "stopVideo")]
    pub fn stop_video();
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
