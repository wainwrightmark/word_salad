use crate::prelude::*;
use bevy::prelude::*;
use capacitor_bindings::admob::*;
use nice_bevy_utils::CanRegisterAsyncEvent;
use strum::EnumIs;

pub struct AdsCommonPlugin;

impl Plugin for AdsCommonPlugin {
    fn build(&self, app: &mut App) {
        app.register_async_event::<AdEvent>();
        app.add_event::<AdRequestEvent>();
        app.init_resource::<AdState>();
        app.init_resource::<InterstitialProgressState>();
    }
}

#[derive(Debug, Event)]
pub enum AdRequestEvent {
    RequestReward {
        event: Option<HintEvent>,
        hints: usize,
    },
    RequestInterstitial,
    RequestConsent,
}

#[derive(Debug, Default, Resource, MavericContext)]
pub struct AdState {
    pub can_show_ads: Option<bool>,
    pub reward_ad: Option<AdLoadInfo>,
    pub interstitial_ad: Option<AdLoadInfo>,
    pub hint_wanted: Option<(usize, Option<HintEvent>)>,
}

#[derive(Debug, Default, Resource, MavericContext)]
pub struct InterstitialProgressState {
    pub levels_without_interstitial: usize,
}

#[derive(Debug, Clone, PartialEq, Event, EnumIs)]
pub enum AdEvent {
    AdsInit,
    // RewardEventsSetUp,
    // RewardEventsSetUpError(String),
    AdsInitError(String),

    FailedToLoadRewardAd(String),
    FailedToShowRewardAd(String),

    FailedToLoadInterstitialAd(String),
    FailedToShowInterstitialAd(String),

    InterstitialLoaded(AdLoadInfo),
    InterstitialShowed,
    RewardAdLoaded(AdLoadInfo),
    RewardAdRewarded(AdMobRewardItem),
}
