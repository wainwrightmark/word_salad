import { registerPlugin } from '../@capacitor/core.js';

var MaxAdContentRating;
(function (MaxAdContentRating) {
    /**
     * Content suitable for general audiences, including families.
     */
    MaxAdContentRating["General"] = "General";
    /**
     * Content suitable for most audiences with parental guidance.
     */
    MaxAdContentRating["ParentalGuidance"] = "ParentalGuidance";
    /**
     * Content suitable for teen and older audiences.
     */
    MaxAdContentRating["Teen"] = "Teen";
    /**
     * Content suitable only for mature audiences.
     */
    MaxAdContentRating["MatureAudience"] = "MatureAudience";
})(MaxAdContentRating || (MaxAdContentRating = {}));

// This enum should be keep in sync with their native equivalents with the same name
var BannerAdPluginEvents;
(function (BannerAdPluginEvents) {
    BannerAdPluginEvents["SizeChanged"] = "bannerAdSizeChanged";
    BannerAdPluginEvents["Loaded"] = "bannerAdLoaded";
    BannerAdPluginEvents["FailedToLoad"] = "bannerAdFailedToLoad";
    /**
     * Open "Adsense" Event after user click banner
     */
    BannerAdPluginEvents["Opened"] = "bannerAdOpened";
    /**
     * Close "Adsense" Event after user click banner
     */
    BannerAdPluginEvents["Closed"] = "bannerAdClosed";
    /**
     * Similarly, this method should be called when an impression is recorded for the ad by the mediated SDK.
     */
    BannerAdPluginEvents["AdImpression"] = "bannerAdImpression";
})(BannerAdPluginEvents || (BannerAdPluginEvents = {}));

/**
 * @see https://developer.android.com/reference/android/widget/LinearLayout#attr_android:gravity
 */
var BannerAdPosition;
(function (BannerAdPosition) {
    /**
     * Banner position be top-center
     */
    BannerAdPosition["TOP_CENTER"] = "TOP_CENTER";
    /**
     * Banner position be center
     */
    BannerAdPosition["CENTER"] = "CENTER";
    /**
     * Banner position be bottom-center(default)
     */
    BannerAdPosition["BOTTOM_CENTER"] = "BOTTOM_CENTER";
})(BannerAdPosition || (BannerAdPosition = {}));

/**
 *  For more information:
 *  https://developers.google.com/admob/ios/banner#banner_sizes
 *  https://developers.google.com/android/reference/com/google/android/gms/ads/AdSize
 *
 * */
var BannerAdSize;
(function (BannerAdSize) {
    /**
     * Mobile Marketing Association (MMA)
     * banner ad size (320x50 density-independent pixels).
     */
    BannerAdSize["BANNER"] = "BANNER";
    /**
     * Interactive Advertising Bureau (IAB)
     * full banner ad size (468x60 density-independent pixels).
     */
    BannerAdSize["FULL_BANNER"] = "FULL_BANNER";
    /**
     * Large banner ad size (320x100 density-independent pixels).
     */
    BannerAdSize["LARGE_BANNER"] = "LARGE_BANNER";
    /**
     * Interactive Advertising Bureau (IAB)
     * medium rectangle ad size (300x250 density-independent pixels).
     */
    BannerAdSize["MEDIUM_RECTANGLE"] = "MEDIUM_RECTANGLE";
    /**
     * Interactive Advertising Bureau (IAB)
     * leaderboard ad size (728x90 density-independent pixels).
     */
    BannerAdSize["LEADERBOARD"] = "LEADERBOARD";
    /**
     * A dynamically sized banner that is full-width and auto-height.
     */
    BannerAdSize["ADAPTIVE_BANNER"] = "ADAPTIVE_BANNER";
    /**
     * @deprecated
     * Will be removed in next AdMob versions use `ADAPTIVE_BANNER`
     * Screen width x 32|50|90
     */
    BannerAdSize["SMART_BANNER"] = "SMART_BANNER";
})(BannerAdSize || (BannerAdSize = {}));

// This enum should be keep in sync with their native equivalents with the same name
var InterstitialAdPluginEvents;
(function (InterstitialAdPluginEvents) {
    /**
     * Emits after trying to prepare and Interstitial, when it is loaded and ready to be show
     */
    InterstitialAdPluginEvents["Loaded"] = "interstitialAdLoaded";
    /**
     * Emits after trying to prepare and Interstitial, when it could not be loaded
     */
    InterstitialAdPluginEvents["FailedToLoad"] = "interstitialAdFailedToLoad";
    /**
     * Emits when the Interstitial ad is visible to the user
     */
    InterstitialAdPluginEvents["Showed"] = "interstitialAdShowed";
    /**
     * Emits when the Interstitial ad is failed to show
     */
    InterstitialAdPluginEvents["FailedToShow"] = "interstitialAdFailedToShow";
    /**
     * Emits when the Interstitial ad is not visible to the user anymore.
     */
    InterstitialAdPluginEvents["Dismissed"] = "interstitialAdDismissed";
})(InterstitialAdPluginEvents || (InterstitialAdPluginEvents = {}));

// This enum should be keep in sync with their native equivalents with the same name
var RewardAdPluginEvents;
(function (RewardAdPluginEvents) {
    /**
     * Emits after trying to prepare a RewardAd and the Video is loaded and ready to be show
     */
    RewardAdPluginEvents["Loaded"] = "onRewardedVideoAdLoaded";
    /**
     * Emits after trying to prepare a RewardAd when it could not be loaded
     */
    RewardAdPluginEvents["FailedToLoad"] = "onRewardedVideoAdFailedToLoad";
    /**
     * Emits when the AdReward video is visible to the user
     */
    RewardAdPluginEvents["Showed"] = "onRewardedVideoAdShowed";
    /**
     * Emits when the AdReward video is failed to show
     */
    RewardAdPluginEvents["FailedToShow"] = "onRewardedVideoAdFailedToShow";
    /**
     * Emits when the AdReward video is not visible to the user anymore.
     *
     * **Important**: This has nothing to do with the reward it self. This event
     * will emits in this two cases:
     * 1. The user starts the video ad but close it before the reward emit.
     * 2. The user start the video and see it until end, then gets the reward
     * and after that the ad is closed.
     */
    RewardAdPluginEvents["Dismissed"] = "onRewardedVideoAdDismissed";
    /**
     * Emits when user get rewarded from AdReward
     */
    RewardAdPluginEvents["Rewarded"] = "onRewardedVideoAdReward";
})(RewardAdPluginEvents || (RewardAdPluginEvents = {}));

/**
 *  For more information:
 *  https://developers.google.com/admob/unity/reference/namespace/google-mobile-ads/ump/api#consentstatus
 *
 * */
var AdmobConsentStatus;
(function (AdmobConsentStatus) {
    /**
     * User consent not required.
     */
    AdmobConsentStatus["NOT_REQUIRED"] = "NOT_REQUIRED";
    /**
     * User consent already obtained.
     */
    AdmobConsentStatus["OBTAINED"] = "OBTAINED";
    /**
     * User consent required but not yet obtained.
     */
    AdmobConsentStatus["REQUIRED"] = "REQUIRED";
    /**
     * Unknown consent status, AdsConsent.requestInfoUpdate needs to be called to update it.
     */
    AdmobConsentStatus["UNKNOWN"] = "UNKNOWN";
})(AdmobConsentStatus || (AdmobConsentStatus = {}));

/**
 *  For more information:
 *  https://developers.google.com/admob/unity/reference/namespace/google-mobile-ads/ump/api#debuggeography
 *
 * */
var AdmobConsentDebugGeography;
(function (AdmobConsentDebugGeography) {
    /**
     * Debug geography disabled.
     */
    AdmobConsentDebugGeography[AdmobConsentDebugGeography["DISABLED"] = 0] = "DISABLED";
    /**
     * Geography appears as in EEA for debug devices.
     */
    AdmobConsentDebugGeography[AdmobConsentDebugGeography["EEA"] = 1] = "EEA";
    /**
     * Geography appears as not in EEA for debug devices.
     */
    AdmobConsentDebugGeography[AdmobConsentDebugGeography["NOT_EEA"] = 2] = "NOT_EEA";
})(AdmobConsentDebugGeography || (AdmobConsentDebugGeography = {}));

const AdMob = registerPlugin('AdMob', {
    web: () => import('../common/web-c8efcf99.js').then(m => new m.AdMobWeb()),
});

export { AdMob, AdmobConsentDebugGeography, AdmobConsentStatus, BannerAdPluginEvents, BannerAdPosition, BannerAdSize, InterstitialAdPluginEvents, MaxAdContentRating, RewardAdPluginEvents };
