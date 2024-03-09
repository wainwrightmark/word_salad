use bevy::prelude::*;
use capacitor_bindings::admob::*;
use nice_bevy_utils::async_event_writer::AsyncEventWriter;
#[cfg(any(feature = "ios", feature = "android"))]
use ws_common::asynchronous;
use ws_common::{ads_common, logging, platform_specific, prelude::*};

use crate::ads::mobile_only::{init_load_and_show_interstitial_ad};

use self::mobile_only::init_load_and_show_reward_ad;

// const MARK_IOS_DEVICE_ID: &str = "d3f1ad44252cdc0f1278cf7347063f07";
// const MARK_ANDROID_DEVICE_ID: &str = "806EEBB5152549F81255DD01CDA931D9";

pub struct AdsPlugin;

impl Plugin for AdsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_everything);
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
            AdRequestEvent::RequestConsent => {
                ws_common::logging::do_or_report_error(reshow_consent_form());
            }

            AdRequestEvent::RequestReward { event, hints } => {
                if ad_state.can_show_ads == Some(true) {
                    ad_state.hint_wanted = Some((*hints, *event));
                    if ad_state.reward_ad.is_some() {
                        ad_state.reward_ad = None;

                        asynchronous::spawn_and_run(mobile_only::try_show_reward_ad(
                            async_writer.clone(),
                        ));
                    } else {
                        asynchronous::spawn_and_run(mobile_only::load_and_show_reward_ad(
                            async_writer.clone(),
                        ));
                    }
                } else {
                    asynchronous::spawn_and_run(init_load_and_show_reward_ad(
                        async_writer.clone(),
                    ));
                }
            }
            AdRequestEvent::RequestInterstitial => {
                if ad_state.can_show_ads == Some(true) {
                    if ad_state.interstitial_ad.is_some() {
                        ad_state.interstitial_ad = None;
                        asynchronous::spawn_and_run(mobile_only::try_show_interstitial_ad(
                            async_writer.clone(),
                        ));
                    } else {
                        asynchronous::spawn_and_run(mobile_only::load_and_show_interstitial_ad(
                            async_writer.clone(),
                        ));
                    }
                } else {
                    asynchronous::spawn_and_run(init_load_and_show_interstitial_ad(
                        async_writer.clone(),
                    ));
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn reshow_consent_form() -> Result<(), capacitor_bindings::error::Error> {
    bevy::log::info!("Resetting consent info");
    Admob::reset_consent_info().await?;

    bevy::log::info!("Re-requesting consent info");

    let consent_info = Admob::request_consent_info(AdmobConsentRequestOptions {
        debug_geography: AdmobConsentDebugGeography::Disabled,
        test_device_identifiers: vec![
            // MARK_ANDROID_DEVICE_ID.to_string(),
            // MARK_IOS_DEVICE_ID.to_string(),
        ],
        tag_for_under_age_of_consent: false,
    })
    .await?;

    info!("Consent Info {consent_info:?}");

    if consent_info.is_consent_form_available && consent_info.status == AdmobConsentStatus::Required
    {
        let _consent_info = Admob::show_consent_form().await?;
    }

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
    purchases: Res<Purchases>,
    current_level: Res<CurrentLevel>,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
) {
    for event in events.read() {
        match event {
            AdEvent::AdsInit => {
                info!("Admob ads initialized");
                if ad_state.can_show_ads != Some(true) {
                    ad_state.can_show_ads = Some(true);

                    if hints.hints_remaining <= INITIAL_HINTS {
                        asynchronous::spawn_and_run(mobile_only::try_load_reward_ad(
                            writer.clone(),
                        ));
                    }

                    if !purchases.remove_ads_purchased {
                        asynchronous::spawn_and_run(mobile_only::try_load_interstitial_ad(
                            writer.clone(),
                        ));
                    }
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

                asynchronous::spawn_and_run(mobile_only::try_load_interstitial_ad(writer.clone()));
                logging::LoggableEvent::InterstitialAdShown.try_log1();
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

                asynchronous::spawn_and_run(mobile_only::try_load_reward_ad(writer.clone()));
                if let Some((hint_count, hint_event)) = ad_state.hint_wanted.take() {
                    hints.hints_remaining += hint_count;
                    hints.total_bought_hints += hint_count;
                    if let Some(hint_event) = hint_event {
                        hint_events.send(hint_event);
                    }

                    platform_specific::show_toast_sync(format!("{hint_count} Hints Rewarded"));
                    logging::LoggableEvent::RewardAdShown.try_log1();
                }
            }
            AdEvent::FailedToLoadRewardAd(s) => {
                platform_specific::show_toast_sync("Failed to load reward ad");
                bevy::log::error!("{s}");
            }
            AdEvent::FailedToShowRewardAd(s) => {
                bevy::log::error!("{s}");

                ad_state.hint_wanted = None;
                platform_specific::show_toast_sync("Failed to show reward ad");

                // #[cfg(any(feature = "ios", feature = "android"))] //TODO request differently
                // {
                //     asynchronous::spawn_and_run(mobile_only::try_load_reward_ad(writer.clone()));
                // }
            }
        }
    }
}

#[allow(unused_variables)]
fn init_everything(writer: AsyncEventWriter<AdEvent>) {
    asynchronous::spawn_and_run(mobile_only::init_everything1_async(writer));
}

#[cfg(feature = "android")]
const BETWEEN_LEVELS_INTERSTITIAL_AD_ID: &str = "ca-app-pub-5238923028364185/8193403915";
#[cfg(feature = "android")]
const BUY_HINTS_REWARD_AD_ID: &str = "ca-app-pub-5238923028364185/7292181940";

#[cfg(all(not(feature = "android"), feature = "ios"))]
const BETWEEN_LEVELS_INTERSTITIAL_AD_ID: &str = "ca-app-pub-5238923028364185/9413401068";
#[cfg(all(not(feature = "android"), feature = "ios"))]
const BUY_HINTS_REWARD_AD_ID: &str = "ca-app-pub-5238923028364185/4515517358";

mod mobile_only {
    use std::{future::Future, time::Duration};

    use super::*;

    ///initialize ads. Returns true if successful
    pub async fn init_everything_async(writer: AsyncEventWriter<AdEvent>) -> bool {
        match try_init_ads_async().await {
            Ok(()) => {
                writer.send_or_panic(AdEvent::AdsInit);
                true
            }
            Err(err) => {
                writer.send_or_panic(AdEvent::AdsInitError(err));
                false
            }
        }
    }

    pub async fn init_everything1_async(writer: AsyncEventWriter<AdEvent>) {
        init_everything_async(writer).await;
    }

    pub async fn try_init_ads_async() -> Result<(), String> {
        Admob::initialize(AdMobInitializationOptions {
            initialize_for_testing: false, //Turn on for testing
            testing_devices: vec![
                // MARK_ANDROID_DEVICE_ID.to_string(),
                // MARK_IOS_DEVICE_ID.to_string(),
            ],
            tag_for_under_age_of_consent: false,
            tag_for_child_directed_treatment: false,
            max_ad_content_rating: MaxAdContentRating::General,
        })
        .await
        .map_err(|x| x.to_string())?;

        let tracking_info = Admob::tracking_authorization_status()
            .await
            .map_err(|x| x.to_string())?;

        match tracking_info.status {
            TrackingAuthorizationStatus::Authorized => {}
            TrackingAuthorizationStatus::Denied => {
                // return Err("Tracking info status Denied".to_string());
            }
            TrackingAuthorizationStatus::NotDetermined => {
                #[allow(deprecated)]
                Admob::request_tracking_authorization()
                    .await
                    .map_err(|x| x.to_string())?;

                Admob::tracking_authorization_status()
                    .await
                    .map_err(|x| x.to_string())?;
            }
            TrackingAuthorizationStatus::Restricted => {
                // return Err("Tracking info status Restricted".to_string());
            }
        };

        let consent_info = Admob::request_consent_info(AdmobConsentRequestOptions {
            debug_geography: AdmobConsentDebugGeography::Disabled,
            test_device_identifiers: vec![
                // MARK_ANDROID_DEVICE_ID.to_string(),
                // MARK_IOS_DEVICE_ID.to_string(),
            ],
            tag_for_under_age_of_consent: false,
        })
        .await
        .map_err(|x| x.to_string())?;

        info!("Consent Info {consent_info:?}");

        if consent_info.is_consent_form_available
            && consent_info.status == AdmobConsentStatus::Required
        {
            let _consent_info = Admob::show_consent_form()
                .await
                .map_err(|x| x.to_string())?;
        }

        Ok(())
    }

    pub async fn try_load_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        let options: AdOptions = AdOptions {
            ad_id: BETWEEN_LEVELS_INTERSTITIAL_AD_ID.to_string(),
            is_testing: false,
            margin: 0.0,
            npa: false,
        };

        match Admob::prepare_interstitial(options).await {
            Ok(load_info) => writer.send_or_panic(AdEvent::InterstitialLoaded(load_info)),
            Err(err) => writer.send_or_panic(AdEvent::FailedToShowInterstitialAd(err.to_string())),
        }
    }

    pub async fn try_show_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        match Admob::show_interstitial().await {
            Ok(()) => writer.send_or_panic(AdEvent::InterstitialShowed),
            Err(err) => writer.send_or_panic(AdEvent::FailedToShowInterstitialAd(err.to_string())),
        }
    }

    pub async fn try_load_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        let options = RewardAdOptions {
            ssv: None,
            ad_id: BUY_HINTS_REWARD_AD_ID.to_string(),
            is_testing: false,
            margin: 0.0,
            npa: false,
        };

        match Admob::prepare_reward_video_ad(options).await {
            Ok(load_info) => writer.send_or_panic(AdEvent::RewardAdLoaded(load_info)),
            Err(err) => writer.send_or_panic(AdEvent::FailedToLoadRewardAd(err.to_string())),
        }
    }

    pub async fn try_show_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        match Admob::show_reward_video_ad().await {
            Ok(item) => writer.send_or_panic(AdEvent::RewardAdRewarded(item)),
            Err(err) => writer.send_or_panic(AdEvent::FailedToShowRewardAd(err.to_string())),
        };
    }

    pub async fn init_load_and_show_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        if init_everything_async(writer.clone()).await {
            load_and_show_reward_ad(writer).await;
        } else {
            writer.send_or_panic(AdEvent::FailedToShowRewardAd(
                "Could not init ads for reward ad".to_string(),
            ));
        }
    }

    pub async fn init_load_and_show_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        if init_everything_async(writer.clone()).await {
            load_and_show_interstitial_ad(writer).await;
        } else {
            writer.send_or_panic(AdEvent::FailedToShowInterstitialAd(
                "Could not init ads for interstitial ad".to_string(),
            ));
        }
    }

    pub async fn load_and_show_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        let options: AdOptions = AdOptions {
            ad_id: BETWEEN_LEVELS_INTERSTITIAL_AD_ID.to_string(),
            is_testing: false,
            margin: 0.0,
            npa: false,
        };

        const INTERSTITIAL_AD_TIMEOUT: f32 = 5.0;

        let prepare_future = Admob::prepare_interstitial(options);

        let timeout_future = with_timeout(
            Duration::from_secs_f32(INTERSTITIAL_AD_TIMEOUT),
            Box::pin(prepare_future),
        )
        .await;

        match timeout_future {
            Ok(Ok(_)) => try_show_interstitial_ad(writer).await,
            Ok(Err(err)) => {
                writer.send_or_panic(AdEvent::FailedToShowInterstitialAd(err.to_string()))
            }
            Err(timeout_err) => {
                writer.send_or_panic(AdEvent::FailedToShowInterstitialAd(timeout_err.to_string()))
            }
        }
    }

    pub async fn load_and_show_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        let options = RewardAdOptions {
            ssv: None,
            ad_id: BUY_HINTS_REWARD_AD_ID.to_string(),
            is_testing: false,
            margin: 0.0,
            npa: false,
        };

        const REWARD_AD_TIMEOUT: f32 = 10.0;

        let prepare_future = Admob::prepare_reward_video_ad(options);

        let timeout_future = with_timeout(
            Duration::from_secs_f32(REWARD_AD_TIMEOUT),
            Box::pin(prepare_future),
        )
        .await;

        match timeout_future {
            Ok(Ok(_)) => {
                try_show_reward_ad(writer).await;
            }
            Ok(Err(err)) => writer.send_or_panic(AdEvent::FailedToLoadRewardAd(err.to_string())),
            Err(timeout_err) => {
                writer.send_or_panic(AdEvent::FailedToLoadRewardAd(timeout_err.to_string()))
            }
        }
    }

    pub fn with_timeout<T>(
        dur: Duration,
        future: T,
    ) -> impl Future<Output = Result<T::Output, async_sleep::timeout::Error>>
    where
        T: Future + Unpin,
    {
        async_sleep::timeout::<async_sleep::AsyncTimerPlatform, T>(dur, future)
    }
}
