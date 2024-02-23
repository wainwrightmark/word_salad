use crate::{all_levels::*, prelude::LevelGroup};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIs, EnumIter};
use ws_core::DesignedLevel;

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    EnumCount,
    EnumIter,
    EnumIs,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Hash,
)]

pub enum LevelSequence {
    USStates,
    EuropeanCountries,
    SouthAndEastAsianCountries,
    MiddleEasternCountries,
    AfricanCountries,

    EuropeanCapitals,
    SouthAndEastAsianCapitals,
    MiddleEasternCapitals,

    Mammals,
    Birds,
    Insects,
    ReptilesAndAmphibians,
    Fruit,
    Vegetables,
    Gemstones,
    Elements,

    NFLTeams,
    NBATeams,
    MLBTeams,
    NHLTeams,
}

impl LevelSequence {
    pub const FIRST: Self = LevelSequence::EuropeanCapitals;

    pub fn get_next(self) -> Option<Self> {
        use LevelSequence::*;
        let r = match self {
            USStates => EuropeanCountries,
            EuropeanCountries => SouthAndEastAsianCountries,
            SouthAndEastAsianCountries => MiddleEasternCountries,
            MiddleEasternCountries => AfricanCountries,
            AfricanCountries => EuropeanCapitals,
            EuropeanCapitals => SouthAndEastAsianCapitals,
            SouthAndEastAsianCapitals => MiddleEasternCapitals,
            MiddleEasternCapitals => Mammals,

            Mammals => Birds,
            Birds => Insects,
            Insects => ReptilesAndAmphibians,
            ReptilesAndAmphibians => Fruit,
            Fruit => Vegetables,
            Vegetables => Gemstones,
            Gemstones => Elements,
            Elements => NFLTeams,

            NFLTeams => NBATeams,
            NBATeams => MLBTeams,
            MLBTeams => NHLTeams,
            NHLTeams => return None,
        };
        return Some(r);
    }

    pub fn group(self) -> LevelGroup {
        use LevelSequence::*;
        match self {
            USStates
            | EuropeanCountries
            | EuropeanCapitals
            | SouthAndEastAsianCountries
            | MiddleEasternCountries
            | SouthAndEastAsianCapitals
            | AfricanCountries
            | MiddleEasternCapitals => LevelGroup::Geography,
            Mammals
            | Birds
            | Insects
            | Fruit
            | Vegetables
            | Gemstones
            | Elements
            | ReptilesAndAmphibians => LevelGroup::NaturalWorld,

            NFLTeams | NBATeams | MLBTeams | NHLTeams => LevelGroup::USSports,
        }
    }

    pub fn get_level(self, index: usize) -> Option<&'static DesignedLevel> {
        let levels = self.levels();

        let index = index;
        levels.get(index)
    }

    pub fn level_count(self) -> usize {
        let levels = self.levels();
        levels.len()
    }

    pub fn free_level_count(self) -> usize {
        2
    }

    pub(crate) fn levels(self) -> &'static Vec<DesignedLevel> {
        let levels = match self {
            LevelSequence::USStates => &*US_STATES,
            LevelSequence::EuropeanCapitals => &*EUROPEAN_CAPITALS,
            LevelSequence::EuropeanCountries => &*EUROPEAN_COUNTRIES,
            LevelSequence::SouthAndEastAsianCountries => &*SOUTH_AND_EAST_ASIAN_COUNTRIES,
            LevelSequence::MiddleEasternCountries => &*MIDDLE_EASTERN_COUNTRIES,
            LevelSequence::SouthAndEastAsianCapitals => &*SOUTH_AND_EAST_ASIAN_CAPITALS,
            LevelSequence::MiddleEasternCapitals => &*MIDDLE_EASTERN_CAPITALS,
            LevelSequence::AfricanCountries => &*AFRICAN_COUNTRIES,

            LevelSequence::Insects => &*INSECTS,
            LevelSequence::Fruit => &*FRUIT,
            LevelSequence::Gemstones => &*GEMSTONES,
            LevelSequence::Vegetables => &*VEGETABLES,
            LevelSequence::Elements => &*ELEMENTS,
            LevelSequence::Mammals => &*MAMMALS,
            LevelSequence::Birds => &*BIRDS,
            LevelSequence::ReptilesAndAmphibians => &*REPTILES_AND_AMPHIBIANS,

            LevelSequence::NFLTeams => &*NFL_TEAMS,
            LevelSequence::NBATeams => &*NBA_TEAMS,
            LevelSequence::MLBTeams => &*MLB_TEAMS,
            LevelSequence::NHLTeams => &*NHL_TEAMS,
        };
        levels
    }

    pub fn name(self) -> &'static str {
        match self {
            LevelSequence::USStates => "US States",
            LevelSequence::EuropeanCapitals => "European Capitals",
            LevelSequence::EuropeanCountries => "European Countries",
            LevelSequence::SouthAndEastAsianCountries => "S & E Asian Countries",
            LevelSequence::AfricanCountries => "African Countries",
            LevelSequence::MiddleEasternCountries => "Middle Eastern Countries",
            LevelSequence::MiddleEasternCapitals => "Middle Eastern Capitals",
            LevelSequence::SouthAndEastAsianCapitals => "S & E Asian Capitals",

            LevelSequence::Insects => "Insects",
            LevelSequence::Gemstones => "Gemstones",
            LevelSequence::Vegetables => "Vegetables",
            LevelSequence::Elements => "Elements",
            LevelSequence::Mammals => "Mammals",
            LevelSequence::Birds => "Birds",
            LevelSequence::Fruit => "Fruit",
            LevelSequence::ReptilesAndAmphibians => "Reptiles & Amphibians",

            LevelSequence::NFLTeams => "NFL Teams",
            LevelSequence::NBATeams => "NBA Teams",
            LevelSequence::MLBTeams => "MLB Teams",
            LevelSequence::NHLTeams => "NHL Teams",
        }
    }
}
