use bevy::prelude::*;
use capacitor_bindings::admob::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
#[cfg(any(feature = "ios", feature = "android"))]
use ws_common::asynchronous;
use ws_common::{ads_common, prelude::*};

pub struct AdsPlugin;

impl Plugin for AdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_ad_events.run_if(|x: EventReader<AdEvent>| !x.is_empty()),
        );

        app.add_systems(
            Update,
            handle_ad_requests.run_if(|x: EventReader<AdRequestEvent>| !x.is_empty()),
        );
    }
}

#[allow(unused_variables)]
#[allow(unused_mut)]
fn handle_ad_requests(
    mut events: EventReader<AdRequestEvent>,
    mut ad_state: ResMut<AdState>,
    mut sync_writer: EventWriter<AdEvent>,
    async_writer: AsyncEventWriter<AdEvent>,
) {
    for event in events.read() {
        match event {
            AdRequestEvent::RequestConsent => {}

            AdRequestEvent::RequestReward { event, hints } => {
                ad_state.hint_wanted = Some((*hints, *event));
                ws_common::platform_specific::show_toast_on_web("We would show a reward ad here");
                sync_writer.send(AdEvent::RewardAdRewarded(AdMobRewardItem {
                    reward_type: "blah".to_string(),
                    amount: 0,
                }));
            }
            AdRequestEvent::RequestInterstitial => {
                ws_common::platform_specific::show_toast_on_web(
                    "We would show an interstitial ad here",
                );
                //sync_writer.send(AdEvent::FailedToShowInterstitialAd("Testing".to_string()));
                sync_writer.send(AdEvent::InterstitialShowed);
            }
        }
    }
}

#[allow(dead_code)]
async fn reshow_consent_form() -> Result<(), capacitor_bindings::error::Error> {
    show_toast_on_web("We would show GDPR");

    Ok(())
}

#[allow(unused_variables)]
fn handle_ad_events(
    mut events: EventReader<AdEvent>,
    mut ad_state: ResMut<AdState>,
    writer: AsyncEventWriter<AdEvent>,
    mut hints: ResMut<HintState>,
    mut interstitial_state: ResMut<InterstitialProgressState>,
    mut hint_events: EventWriter<HintEvent>,
    current_level: Res<CurrentLevel>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    for event in events.read() {
        match event {
            AdEvent::AdsInit => {
                info!("Admob ads initialized");
                if ad_state.can_show_ads != Some(true) {
                    ad_state.can_show_ads = Some(true);
                }
            }
            AdEvent::AdsInitError(err) => {
                ad_state.can_show_ads = Some(false);
                bevy::log::error!(err);
            }
            AdEvent::InterstitialLoaded(i) => {
                ad_state.interstitial_ad = Some(i.clone());
            }
            AdEvent::FailedToLoadInterstitialAd(err) => {
                ads_common::on_interstitial_failed(&current_level, &mut change_level_events);
                bevy::log::error!("{}", err);
            }
            AdEvent::InterstitialShowed => {
                ads_common::on_interstitial_showed(
                    &mut interstitial_state,
                    &current_level,
                    &mut change_level_events,
                );
            }
            AdEvent::FailedToShowInterstitialAd(err) => {
                ads_common::on_interstitial_failed(&current_level, &mut change_level_events);
                bevy::log::error!("{}", err);
            }

            AdEvent::RewardAdLoaded(ad) => {
                info!("Admob reward ad loaded");
                ad_state.reward_ad = Some(ad.clone())
            }
            AdEvent::RewardAdRewarded(reward) => {
                info!("admob Reward ad rewarded {reward:?}",);
                if let Some((hint_count, hint_event)) = ad_state.hint_wanted.take() {
                    hints.hints_remaining += hint_count;
                    hints.total_bought_hints += hint_count;
                    if let Some(hint_event) = hint_event {
                        hint_events.send(hint_event);
                    }
                }
            }
            AdEvent::FailedToLoadRewardAd(s) => {
                bevy::log::error!("{s}");
            }
            AdEvent::FailedToShowRewardAd(s) => {
                bevy::log::error!("{s}");

                ad_state.hint_wanted = None;
            }
        }
    }
}
