use strum::Display;
use ws_core::layout::entities::SelfieMode;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum HapticEvent {
    UseHint,
    FinishWord,
    FinishPuzzle,
}

impl HapticEvent {
    pub fn try_activate(&self, selfie_mode: SelfieMode) {
        if selfie_mode.is_selfie_mode {}
        #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios", feature = "web")) )]
        {
            //bevy::log::info!("Haptic event {self}");
            use capacitor_bindings::haptics::{ImpactOptions, ImpactStyle};
            match self {
                HapticEvent::UseHint => {
                    crate::logging::do_or_report_error(
                        capacitor_bindings::haptics::Haptics::impact(ImpactOptions {
                            style: capacitor_bindings::haptics::ImpactStyle::Light,
                        }),
                    );
                }
                HapticEvent::FinishWord => {
                    crate::logging::do_or_report_error(
                        capacitor_bindings::haptics::Haptics::impact(ImpactOptions {
                            style: ImpactStyle::Light,
                        }),
                    );
                }
                HapticEvent::FinishPuzzle => {
                    crate::logging::do_or_report_error(
                        capacitor_bindings::haptics::Haptics::impact(ImpactOptions {
                            style: ImpactStyle::Heavy,
                        }),
                    );
                }
            }
        }
    }
}
