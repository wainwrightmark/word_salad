use crate::{asynchronous, prelude::*};
use bevy::prelude::*;
use capacitor_bindings::admob::{self, *};
use nice_bevy_utils::{async_event_writer::AsyncEventWriter, CanRegisterAsyncEvent};
use strum::EnumIs;

pub struct AdsPlugin;

impl Plugin for AdsPlugin {
    fn build(&self, app: &mut App) {
        app.register_async_event::<AdEvent>();
        app.add_event::<AdRequestEvent>();
        app.add_systems(Startup, init_everything);
        app.init_resource::<AdState>();

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
    RequestReward,
}

#[derive(Debug, Default, Resource, MavericContext)]
pub struct AdState {
    pub can_show_ads: Option<bool>,

    pub rewards_ads_set_up: Option<bool>,
    pub reward_ad: Option<AdLoadInfo>,
}

fn handle_ad_requests(
    mut events: EventReader<AdRequestEvent>,
    ad_state: Res<AdState>,
    writer: AsyncEventWriter<AdEvent>,
) {
    for event in events.read() {
        match event {
            AdRequestEvent::RequestReward => {
                if ad_state.can_show_ads == Some(true) {
                    if ad_state.rewards_ads_set_up == Some(true) {
                        if ad_state.reward_ad.is_some() {
                            asynchronous::spawn_and_run(try_show_reward_ad(writer.clone()));
                        } else {
                            warn!("Cannot request reward with admob (no reward ad is loaded)")
                        }
                    } else {
                        warn!("Cannot request reward with admob (rewards are not set up)")
                    }
                } else {
                    warn!("Cannot request reward with admob (app cannot show ads)")
                }
            }
        }
    }
}

fn handle_ad_events(
    mut events: EventReader<AdEvent>,
    mut ad_state: ResMut<AdState>,
    writer: AsyncEventWriter<AdEvent>,
    mut hints: ResMut<HintState>,
) {
    for event in events.read() {
        match event {
            AdEvent::AdsInit => {
                info!("Admob ads initialized");
                if ad_state.can_show_ads != Some(true) {
                    ad_state.can_show_ads = Some(true);
                    if ad_state.rewards_ads_set_up == Some(true) {
                        asynchronous::spawn_and_run(try_load_reward_ad(writer.clone()));
                    }
                }
            }
            AdEvent::AdsInitError(err) => {
                ad_state.can_show_ads = Some(false);
                bevy::log::error!(err);
            }

            AdEvent::RewardEventsSetUp => {
                info!("Admob reward events set up");
                if ad_state.rewards_ads_set_up != Some(true) {
                    ad_state.rewards_ads_set_up = Some(true);
                    if ad_state.can_show_ads == Some(true) {
                        asynchronous::spawn_and_run(try_load_reward_ad(writer.clone()));
                    }
                }
            }
            AdEvent::RewardEventsSetUpError(err) => {
                ad_state.rewards_ads_set_up = Some(false);
                bevy::log::error!(err);
            }

            AdEvent::InterstitialLoaded(_) => {}
            AdEvent::InterstitialFailedToLoad(err) => {
                bevy::log::error!("{}", err.message);
            }
            AdEvent::InterstitialShowed => {}
            AdEvent::InterstitialFailedToShow(err) => {
                bevy::log::error!("{}", err.message);
            }
            AdEvent::InterstitialAdDismissed => {}

            AdEvent::RewardFailedToLoad(err) => {
                bevy::log::error!("{}", err.message);
            }
            AdEvent::RewardAdLoaded(ad) => {
                info!("Admob reward ad loaded");
                ad_state.reward_ad = Some(ad.clone())
            }
            AdEvent::RewardAdRewarded(reward) => {
                info!("admob Reward ad rewarded {reward:?}",);
                hints.hints_remaining += 5;
                hints.total_bought_hints += 5;
            }
            AdEvent::RewardAdDismissed => {
                ad_state.reward_ad = None;
                asynchronous::spawn_and_run(try_load_reward_ad(writer.clone()));
            }
            AdEvent::RewardAdFailedToShow(err) => {
                bevy::log::error!("{}", err.message);
            }
            AdEvent::RewardShowed => {
                info!("Reward Showed");
            }
            AdEvent::OtherError(s) => {
                bevy::log::error!("{s}");
            }
        }
    }
}

fn init_everything(writer: AsyncEventWriter<AdEvent>) {
    asynchronous::spawn_and_run(init_everything_async(writer));
}

#[derive(Debug, Clone, PartialEq, Event, EnumIs)]
pub enum AdEvent {
    AdsInit,
    RewardEventsSetUp,
    RewardEventsSetUpError(String),
    AdsInitError(String),
    OtherError(String),

    InterstitialLoaded(AdLoadInfo),
    InterstitialFailedToLoad(AdMobError),
    InterstitialShowed,
    InterstitialFailedToShow(AdMobError),
    InterstitialAdDismissed,

    RewardFailedToLoad(AdMobError),
    RewardAdLoaded(AdLoadInfo),
    RewardAdRewarded(AdMobRewardItem),
    RewardAdDismissed,
    RewardAdFailedToShow(AdMobError),
    RewardShowed,
}

async fn init_everything_async(writer: AsyncEventWriter<AdEvent>) {
    match try_init_ads_async().await {
        Ok(()) => {
            writer.send_async(AdEvent::AdsInit).await.unwrap();
        }
        Err(err) => writer.send_async(AdEvent::AdsInitError(err)).await.unwrap(),
    }

    match set_up_reward_events(writer.clone()).await {
        Ok(()) => {
            writer.send_async(AdEvent::RewardEventsSetUp).await.unwrap();
        }
        Err(err) => writer
            .send_async(AdEvent::RewardEventsSetUpError(err.to_string()))
            .await
            .unwrap(),
    }
}

async fn try_init_ads_async() -> Result<(), String> {
    admob::Admob::initialize(AdMobInitializationOptions {
        initialize_for_testing: true,
        testing_devices: vec![],
        tag_for_under_age_of_consent: false,
        tag_for_child_directed_treatment: false,
        max_ad_content_rating: MaxAdContentRating::General,
    })
    .await
    .map_err(|x| x.to_string())?;

    let tracking_info = admob::Admob::tracking_authorization_status()
        .await
        .map_err(|x| x.to_string())?;

    let tracking_info = match tracking_info.status {
        TrackingAuthorizationStatus::Authorized => tracking_info,
        TrackingAuthorizationStatus::Denied => {
            return Err("Tracking info status Denied".to_string());
        }
        TrackingAuthorizationStatus::NotDetermined => {
            #[allow(deprecated)]
            admob::Admob::request_tracking_authorization()
                .await
                .map_err(|x| x.to_string())?;

            let tracking_info = admob::Admob::tracking_authorization_status()
                .await
                .map_err(|x| x.to_string())?;

            tracking_info
        }
        TrackingAuthorizationStatus::Restricted => {
            return Err("Tracking info status Restricted".to_string());
        }
    };

    if tracking_info.status != TrackingAuthorizationStatus::Authorized {
        return Err(format!("Tracking info status {:?}", tracking_info.status));
    }

    // let consent_info = admob::Admob::request_consent_info(AdmobConsentRequestOptions {
    //     debug_geography: AdmobConsentDebugGeography::Disabled,
    //     test_device_identifiers: vec![],
    //     tag_for_under_age_of_consent: false,
    // })
    // .await
    // .map_err(|x| x.to_string())?;

    // if consent_info.is_consent_form_available
    //     && consent_info.status == AdmobConsentStatus::Required
    // {
    //     let consent_info = admob::Admob::show_consent_form()
    //         .await
    //         .map_err(|x| x.to_string())?;
    //     if consent_info.status == AdmobConsentStatus::Required {
    //         return Err("Consent info still required".to_string());
    //     } else if consent_info.status == AdmobConsentStatus::Unknown {
    //         return Err("Consent info unknown".to_string());
    //     }
    // }

    Ok(())
}

async fn set_up_interstitial_events(
    writer: AsyncEventWriter<AdEvent>,
) -> Result<(), capacitor_bindings::error::Error> {
    {
        let writer = writer.clone();
        Admob::add_interstitial_ad_dismissed_listener(move |_| {
            writer
                .send_blocking(AdEvent::InterstitialAdDismissed)
                .unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_interstitial_failed_to_load_listener(move |e| {
            writer
                .send_blocking(AdEvent::InterstitialFailedToLoad(e))
                .unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_interstitial_failed_to_show_listener(move |e| {
            writer
                .send_blocking(AdEvent::InterstitialFailedToShow(e))
                .unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_interstitial_showed_listener(move |_| {
            writer.send_blocking(AdEvent::InterstitialShowed).unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_interstitial_ad_loaded_listener(move |i| {
            writer
                .send_blocking(AdEvent::InterstitialLoaded(i))
                .unwrap()
        })
        .await?
        .leak();
    }

    Ok(())
}

async fn set_up_reward_events(
    writer: AsyncEventWriter<AdEvent>,
) -> Result<(), capacitor_bindings::error::Error> {
    {
        let writer = writer.clone();
        Admob::add_reward_ad_dismissed_listener(move |_| {
            writer.send_blocking(AdEvent::RewardAdDismissed).unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_reward_ad_failed_to_show_listener(move |e| {
            writer
                .send_blocking(AdEvent::RewardAdFailedToShow(e))
                .unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_reward_ad_loaded_listener(move |e| {
            writer.send_blocking(AdEvent::RewardAdLoaded(e)).unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_reward_ad_rewarded_listener(move |i| {
            info!("admob rewarded listener");
            writer.send_blocking(AdEvent::RewardAdRewarded(i)).unwrap()
        })
        .await?
        .leak();
    }
    {
        let writer = writer.clone();
        Admob::add_reward_failed_to_load_listener(move |e| {
            writer
                .send_blocking(AdEvent::RewardFailedToLoad(e))
                .unwrap()
        })
        .await?
        .leak();
    }

    Ok(())
}

async fn try_load_reward_ad(writer: AsyncEventWriter<AdEvent>) {
    let options = RewardAdOptions {
        ssv: None,
        ad_id: BUY_HINTS_REWARD_AD_ID.to_string(),
        is_testing: true,
        margin: 10.0,
        npa: false,
    };

    match Admob::prepare_reward_video_ad(options).await {
        Ok(load_info) => writer
            .send_async(AdEvent::RewardAdLoaded(load_info))
            .await
            .unwrap(),
        Err(err) => writer
            .send_async(AdEvent::OtherError(err.to_string()))
            .await
            .unwrap(),
    }
}

pub async fn try_show_reward_ad(writer: AsyncEventWriter<AdEvent>) {
    match admob::Admob::show_reward_video_ad().await {
        Ok(item) => writer
            .send_async(AdEvent::RewardAdRewarded(item))
            .await
            .unwrap(),
        Err(err) => writer
            .send_async(AdEvent::OtherError(err.to_string()))
            .await
            .unwrap(),
    };
}

// pub async fn load_between_levels_interstitial(){
//     let load_options = Admob::prepare_interstitial(AdOptions {
//         ad_id: BETWEEN_LEVELS_INTERSTITIAL_AD_ID.to_string(),
//         is_testing: true,
//         margin: 10.0,
//         npa: false,
//     })
//     .await
//     .map_err(|x| x.to_string())?;
// }

// pub async fn show_interstitial() -> Result<(), String> {
//     Admob::show_interstitial()
//         .await
//         .map_err(|x| x.to_string())?;

//     Ok(())
// }

const BETWEEN_LEVELS_INTERSTITIAL_AD_ID: &'static str = "ca-app-pub-5238923028364185/8193403915";
const BUY_HINTS_REWARD_AD_ID: &'static str = "ca-app-pub-5238923028364185/7292181940";
