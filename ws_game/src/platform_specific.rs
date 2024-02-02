pub fn show_toast_on_web(options: impl Into<capacitor_bindings::toast::ShowOptions> + 'static) {
    #[cfg(feature = "web")]
    {
        crate::logging::do_or_report_error(capacitor_bindings::toast::Toast::show(options));
    }
}

pub fn request_review() {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        crate::logging::do_or_report_error(capacitor_bindings::rate::Rate::request_review());
    }
    show_toast_on_web("We would request app review here");
}
