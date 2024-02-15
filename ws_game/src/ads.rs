#[cfg(any(feature = "ios", feature = "android"))]
use crate::asynchronous;
use crate::prelude::*;
use bevy::prelude::*;
use capacitor_bindings::admob::*;
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};
use strum::EnumIs;

pub struct AdsPlugin;

impl Plugin for AdsPlugin {
    fn build(&self, app: &mut App) {
        app.register_async_event::<AdEvent>();
        app.add_event::<AdRequestEvent>();
        app.add_systems(Startup, init_everything);
        app.init_resource::<AdState>();
        app.init_resource::<InterstitialProgressState>();

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

#[allow(unused_variables)]
#[allow(unused_mut)]
fn handle_ad_requests(
    mut events: EventReader<AdRequestEvent>,
    mut ad_state: ResMut<AdState>,
    writer: AsyncEventWriter<AdEvent>,
) {
    for event in events.read() {
        match event {
            AdRequestEvent::RequestConsent => {
                #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
                crate::logging::do_or_report_error(reshow_consent_form());
            }

            AdRequestEvent::RequestReward { event, hints } => {
                #[cfg(any(feature = "ios", feature = "android"))]
                {
                    if ad_state.can_show_ads == Some(true) {
                        if ad_state.reward_ad.is_some() {
                            ad_state.reward_ad = None;
                            ad_state.hint_wanted = Some((*hints, *event));
                            asynchronous::spawn_and_run(mobile_only::try_show_reward_ad(
                                writer.clone(),
                            ));
                        } else {
                            warn!("Cannot request reward with admob (no reward ad is loaded)")
                        }
                    } else {
                        warn!("Cannot request reward with admob (ads are not set up)")
                    }
                }

                #[cfg(not(any(feature = "ios", feature = "android")))]
                {
                    ad_state.hint_wanted = Some((*hints, *event));
                    crate::platform_specific::show_toast_on_web("We would show a reward ad here");
                    writer
                        .send_blocking(AdEvent::RewardAdRewarded(AdMobRewardItem {
                            reward_type: "blah".to_string(),
                            amount: 0,
                        }))
                        .unwrap();
                }
            }
            AdRequestEvent::RequestInterstitial => {
                #[cfg(any(feature = "ios", feature = "android"))]
                {
                    if ad_state.can_show_ads == Some(true) {
                        if ad_state.interstitial_ad.is_some() {
                            ad_state.interstitial_ad = None;
                            asynchronous::spawn_and_run(mobile_only::try_show_interstitial_ad(
                                writer.clone(),
                            ));
                        } else {
                            warn!("Cannot request interstitial with admob (ads are not set up)")
                        }
                    } else {
                        warn!("Cannot request interstitial with admob (ads are not set up)")
                    }
                }
                #[cfg(not(any(feature = "ios", feature = "android")))]
                {
                    crate::platform_specific::show_toast_on_web(
                        "We would show an interstitial ad here",
                    );
                    writer.send_blocking(AdEvent::InterstitialShowed).unwrap();
                }
            }
        }
    }
}

#[allow(dead_code)]
async fn reshow_consent_form() -> Result<(), capacitor_bindings::error::Error> {
    #[cfg(feature = "android")] //todo also ios
    {
        let _r = Admob::show_consent_form().await?;
    }
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
) {
    for event in events.read() {
        match event {
            AdEvent::AdsInit => {
                info!("Admob ads initialized");
                if ad_state.can_show_ads != Some(true) {
                    ad_state.can_show_ads = Some(true);
                    #[cfg(any(feature = "ios", feature = "android"))]
                    {
                        asynchronous::spawn_and_run(mobile_only::try_load_reward_ad(
                            writer.clone(),
                        ));
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
                bevy::log::error!("{}", err);
            }
            AdEvent::InterstitialShowed => {
                interstitial_state.levels_without_interstitial = 0;
                #[cfg(any(feature = "ios", feature = "android"))]
                {
                    asynchronous::spawn_and_run(mobile_only::try_load_interstitial_ad(
                        writer.clone(),
                    ));
                }
            }
            AdEvent::FailedToShowInterstitialAd(err) => {
                bevy::log::error!("{}", err);
                // #[cfg(any(feature = "ios", feature = "android"))] //todo fix
                // {
                //     asynchronous::spawn_and_run(mobile_only::try_load_interstitial_ad(
                //         writer.clone(),
                //     ));
                // }
            }

            AdEvent::RewardAdLoaded(ad) => {
                info!("Admob reward ad loaded");
                ad_state.reward_ad = Some(ad.clone())
            }
            AdEvent::RewardAdRewarded(reward) => {
                info!("admob Reward ad rewarded {reward:?}",);

                #[cfg(any(feature = "ios", feature = "android"))]
                {
                    asynchronous::spawn_and_run(mobile_only::try_load_reward_ad(writer.clone()));
                }
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
    #[cfg(any(feature = "ios", feature = "android"))]
    {
        asynchronous::spawn_and_run(mobile_only::init_everything_async(writer));
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

#[allow(dead_code)]
const BETWEEN_LEVELS_INTERSTITIAL_AD_ID: &'static str = "ca-app-pub-5238923028364185/8193403915"; //todo different on ios
#[allow(dead_code)]
const BUY_HINTS_REWARD_AD_ID: &'static str = "ca-app-pub-5238923028364185/7292181940"; //todo different on ios

//const HINTS_REWARD_AMOUNT: usize = 5;

#[cfg(any(feature = "ios", feature = "android"))]
mod mobile_only {
    use super::*;

    pub async fn init_everything_async(writer: AsyncEventWriter<AdEvent>) {
        match try_init_ads_async().await {
            Ok(()) => {
                writer.send_async(AdEvent::AdsInit).await.unwrap();
            }
            Err(err) => writer.send_async(AdEvent::AdsInitError(err)).await.unwrap(),
        }
    }

    pub async fn try_init_ads_async() -> Result<(), String> {
        Admob::initialize(AdMobInitializationOptions {
            initialize_for_testing: true,
            testing_devices: vec!["806EEBB5152549F81255DD01CDA931D9".to_string()],
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

        #[cfg(any(feature = "android"))]
        {
            let consent_info = Admob::request_consent_info(AdmobConsentRequestOptions {
                debug_geography: AdmobConsentDebugGeography::Disabled,
                test_device_identifiers: vec!["806EEBB5152549F81255DD01CDA931D9".to_string()],
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
                // if consent_info.status == AdmobConsentStatus::Required {
                //     return Err("Consent info still required".to_string());
                // } else if consent_info.status == AdmobConsentStatus::Unknown {
                //     return Err("Consent info unknown".to_string());
                // }
            }
        }

        Ok(())
    }

    pub async fn try_load_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        let options: AdOptions = AdOptions {
            ad_id: BETWEEN_LEVELS_INTERSTITIAL_AD_ID.to_string(),
            is_testing: true,
            margin: 0.0,
            npa: false,
        };

        match Admob::prepare_interstitial(options).await {
            Ok(load_info) => writer
                .send_async(AdEvent::InterstitialLoaded(load_info))
                .await
                .unwrap(),
            Err(err) => writer
                .send_async(AdEvent::FailedToShowInterstitialAd(err.to_string()))
                .await
                .unwrap(),
        }
    }

    pub async fn try_show_interstitial_ad(writer: AsyncEventWriter<AdEvent>) {
        match Admob::show_interstitial().await {
            Ok(()) => writer
                .send_async(AdEvent::InterstitialShowed)
                .await
                .unwrap(),
            Err(err) => writer
                .send_async(AdEvent::FailedToShowInterstitialAd(err.to_string()))
                .await
                .unwrap(),
        }
    }

    pub async fn try_load_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        let options = RewardAdOptions {
            ssv: None,
            ad_id: BUY_HINTS_REWARD_AD_ID.to_string(),
            is_testing: true,
            margin: 0.0,
            npa: false,
        };

        match Admob::prepare_reward_video_ad(options).await {
            Ok(load_info) => writer
                .send_async(AdEvent::RewardAdLoaded(load_info))
                .await
                .unwrap(),
            Err(err) => writer
                .send_async(AdEvent::FailedToLoadRewardAd(err.to_string()))
                .await
                .unwrap(),
        }
    }

    pub async fn try_show_reward_ad(writer: AsyncEventWriter<AdEvent>) {
        match Admob::show_reward_video_ad().await {
            Ok(item) => writer
                .send_async(AdEvent::RewardAdRewarded(item))
                .await
                .unwrap(),
            Err(err) => writer
                .send_async(AdEvent::FailedToShowRewardAd(err.to_string()))
                .await
                .unwrap(),
        };
    }
}
