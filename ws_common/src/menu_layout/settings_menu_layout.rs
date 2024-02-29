use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{layout::entities::*, LayoutStructureWithTextOrImage};

use crate::achievements::UserSignedIn;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum SettingsLayoutEntity {
    AdsConsent,
    RestorePurchases,
    SeeAchievements,
    SyncAchievements,

}

impl SettingsLayoutEntity {
    fn only_when_signed_in(&self) -> bool {
        match self {
            SettingsLayoutEntity::AdsConsent => false,
            SettingsLayoutEntity::SeeAchievements => true,
            SettingsLayoutEntity::SyncAchievements => true,
            SettingsLayoutEntity::RestorePurchases => false,
        }
    }
}

impl MenuButtonsLayout for SettingsLayoutEntity {
    type Context = UserSignedIn;

    fn index(&self) -> usize {
        *self as usize
    }

    fn count(_context: &Self::Context) -> usize {
        Self::COUNT
    }

    fn iter_all(context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter().filter(|x| !x.only_when_signed_in() || context.is_signed_in)
    }
}

impl LayoutStructureWithTextOrImage for SettingsLayoutEntity {
    fn text_or_image(&self, _context: &Self::Context<'_>) -> ws_core::prelude::TextOrImage {
        match self {
            SettingsLayoutEntity::AdsConsent => ws_core::TextOrImage::Text {
                text: "Manage Ads Consent",
            },
            SettingsLayoutEntity::SeeAchievements => ws_core::TextOrImage::Text {
                text: "See Achievements",
            },

            SettingsLayoutEntity::SyncAchievements => ws_core::TextOrImage::Text {
                text: "Sync Achievements",
            },

            SettingsLayoutEntity::RestorePurchases => ws_core::TextOrImage::Text {
                text: "Restore Purchases",
            },
        }
    }
}
