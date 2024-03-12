pub fn go_to_app_store() {
    #[cfg(target_arch = "wasm32")]
    crate::asynchronous::spawn_and_run(go_to_app_store_async());
}

#[cfg(target_arch = "wasm32")]
async fn go_to_app_store_async() {
    let device_result = capacitor_bindings::device::Device::get_info().await;

    const DEFAULT_URL: &str = "https://bleppo.co.uk/games/";
    const IOS_URL: &str = "https://bleppo.co.uk/games/";
    const ANDROID_URL: &str = "https://play.google.com/store/apps/details?id=com.wordsalad.app";

    let url: &'static str = match device_result {
        Ok(d) => match d.operating_system {
            capacitor_bindings::device::OperatingSystem::IOs => IOS_URL,
            capacitor_bindings::device::OperatingSystem::Android => ANDROID_URL,
            capacitor_bindings::device::OperatingSystem::Windows => DEFAULT_URL,
            capacitor_bindings::device::OperatingSystem::Mac => IOS_URL,
            capacitor_bindings::device::OperatingSystem::Unknown => DEFAULT_URL,
        },
        Err(e) => {
            bevy::log::error!("{e}");
            DEFAULT_URL
        }
    };

    crate::wasm::open_link(url);
}
