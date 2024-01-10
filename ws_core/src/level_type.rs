use strum::EnumIs;

#[derive(Debug, Clone, Copy,  PartialEq, Eq, EnumIs)]
pub enum LevelType{
    Tutorial,
    Fixed,
    DailyChallenge,
    Custom,
    NonLevel
}