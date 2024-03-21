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

        app.add_systems(Update, countdown_failed_ad.run_if(is_ad_failed));
    }
}

pub const AD_FAILED_SECONDS: i64 = 5i64;

fn is_ad_failed(current_level: Res<CurrentLevel>) -> bool {
    match current_level.as_ref() {
        CurrentLevel::NonLevel(nl) => nl.is_ad_failed(),
        _ => false,
    }
}

fn countdown_failed_ad(mut current_level: ResMut<CurrentLevel>, mut redraw: ResMut<RedrawMarker>, mut ips: ResMut<InterstitialProgressState>) {

    if let CurrentLevel::NonLevel(NonLevel::AdFailed {  since: Some(since), next_level }) = current_level.as_ref() {

        if chrono::Utc::now().signed_duration_since(since).num_seconds() >= AD_FAILED_SECONDS{
            *current_level = CurrentLevel::NonLevel(NonLevel::AdFailed { next_level: *next_level, since: None });
            ips.levels_without_interstitial = 0;
        }
        redraw.set_changed()

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

#[derive(Debug, Resource, MavericContext)]
pub struct InterstitialProgressState {
    pub levels_without_interstitial: usize,
}

impl Default for InterstitialProgressState {
    fn default() -> Self {
        Self {
            levels_without_interstitial: 1,
        }
    }
}

pub fn on_interstitial_showed(
    ips: &mut ResMut<InterstitialProgressState>,
    current_level: &CurrentLevel,
    change_level_events: &mut EventWriter<ChangeLevelEvent>,
) {
    ips.levels_without_interstitial = 0;
    if let CurrentLevel::NonLevel(NonLevel::AdBreak(next_level)) = current_level {
        let next_level: CurrentLevel = (*next_level).into();
        change_level_events.send(ChangeLevelEvent::ChangeTo(next_level));
    }
}

pub fn on_interstitial_failed(
    current_level: &CurrentLevel,
    change_level_events: &mut EventWriter<ChangeLevelEvent>,
) {
    if let CurrentLevel::NonLevel(NonLevel::AdBreak(next_level)) = current_level {
        change_level_events.send(ChangeLevelEvent::ChangeTo(CurrentLevel::NonLevel(
            NonLevel::AdFailed {
                next_level: *next_level,
                since: Some(chrono::Utc::now()) ,
            },
        )));
    }
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
