use bevy::log::error;

#[allow(unused_variables)]
pub fn show_toast_on_web(options: impl Into<capacitor_bindings::toast::ShowOptions> + 'static) {
    #[cfg(feature = "web")]
    {
        crate::logging::do_or_report_error(capacitor_bindings::toast::Toast::show(options));
    }
}

#[allow(unused_variables)]
pub async fn show_toast_async(options: impl Into<capacitor_bindings::toast::ShowOptions> + 'static) {
    match capacitor_bindings::toast::Toast::show(options).await{
        Ok(_) => {},
        Err(err) => {
            error!("{err}")
        },
    }
}

pub fn request_review() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        crate::logging::do_or_report_error(capacitor_bindings::rate::Rate::request_review());
    }
    show_toast_on_web("We would request app review here");
}


pub fn submit_score(data: crate::compatibility::SubmitScoreData) {
    bevy::log::info!("Submitting Score {data:?}");
    #[cfg(any(feature = "ios"))] //only on IOS as android don't do recurring leaderboards
    {
        crate::logging::do_or_report_error(capacitor_bindings::game_connect::GameConnect::submit_score(
            <crate::compatibility::SubmitScoreData as Into<capacitor_bindings::game_connect::SubmitScoreOptions>>::into(data)

        ));
    }
}