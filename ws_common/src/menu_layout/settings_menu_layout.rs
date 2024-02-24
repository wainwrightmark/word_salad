
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use ws_core::{
    layout::entities::*,
    LayoutStructureWithTextOrImage,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Display, EnumIter, EnumCount,
)]
pub enum SettingsLayoutEntity {
    AdsConsent,
    SeeAchievements,
    //SyncAchievements  //todo
    RestorePurchases  //todo
}

impl MenuButtonsLayout for SettingsLayoutEntity {
    type Context = ();

    fn index(&self) -> usize {
        *self as usize
    }

    fn count(_context: &Self::Context) -> usize {
        Self::COUNT
    }

    fn iter_all(_context: &Self::Context) -> impl Iterator<Item = Self> {
        Self::iter()
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

            SettingsLayoutEntity::RestorePurchases => ws_core::TextOrImage::Text {
                text: "Restore Purchases",
            },
        }
    }
}
