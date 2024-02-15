use crate::prelude::*;
use bevy::{prelude::*, utils::HashSet};
use nice_bevy_utils::{
    async_event_writer::AsyncEventWriter, CanInitTrackedResource, CanRegisterAsyncEvent,
    TrackableResource,
};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use strum::{AsRefStr, Display};

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sign_in_user);
        app.init_resource::<UserSignedIn>();
        app.register_async_event::<SignInEvent>();
        app.add_systems(Update, check_for_sign_in);

        app.init_tracked_resource::<AchievementsState>();
        app.init_tracked_resource::<WordsFoundCountState>();
        app.add_systems(
            Update,
            track_level_completion_achievements.run_if(|found_words: Res<FoundWordsState>| {
                found_words.is_changed() || found_words.is_level_complete()
            }),
        );
        app.add_systems(
            Update,
            track_found_words.run_if(|e: EventReader<WordFoundEvent>| !e.is_empty()),
        );

        app.add_systems(
            Update,
            track_hint_achievements.run_if(|e: EventReader<HintEvent>| !e.is_empty()),
        );

        app.add_systems(
            Update,
            track_streak_achievements.run_if(|s: Res<Streak>| s.is_changed()),
        );

        app.add_systems(
            Update,
            track_selfie_achievements.run_if(|s: Res<VideoResource>| s.is_changed()),
        );
    }
}

#[derive(Debug, Resource, Default, Clone, PartialEq, Eq, MavericContext)]
pub struct UserSignedIn {
    pub is_signed_in: bool,
}

#[derive(Debug, Event, Clone, Copy, Eq, PartialEq)]
pub struct SignInEvent;

fn check_for_sign_in(
    mut ev: EventReader<SignInEvent>,
    mut signed_in: ResMut<UserSignedIn>,
    achievements: Res<AchievementsState>,
) {
    for _ in ev.read() {
        signed_in.is_signed_in = true;
        achievements.resync();
    }
}

#[allow(unused_variables)]
fn sign_in_user(writer: AsyncEventWriter<SignInEvent>) {
    #[cfg(any(feature = "android", feature = "ios"))]
    {
        async fn sign_in_async(
            writer: AsyncEventWriter<SignInEvent>,
        ) -> Result<(), capacitor_bindings::error::Error> {
            let user = capacitor_bindings::game_connect::GameConnect::sign_in().await?;
            info!("User signed in: {user:?}");
            let _ = writer.send_async(SignInEvent).await;

            Ok(())
        }

        spawn_and_run(async move {
            match sign_in_async(writer).await {
                Ok(()) => {}
                Err(err) => error!("{err}"),
            }
        });
    }
}

pub fn show_achievements() {
    info!("Showing achievements");
    #[cfg(any(feature = "android", feature = "ios"))]
    {

        use capacitor_bindings::game_connect::*;
        do_or_report_error(GameConnect::show_achievements());
    }
    crate::platform_specific::show_toast_on_web("We would go to achievement page");
}

fn track_hint_achievements(
    mut achievements: ResMut<AchievementsState>,
    mut events: EventReader<HintEvent>,
    current_level: Res<CurrentLevel>,
) {
    for _ in events.read() {
        if current_level.should_spend_hints() {
            maybe_unlock(&mut achievements, Achievement::QAndA);
        }
    }
}

fn track_selfie_achievements(
    mut achievements: ResMut<AchievementsState>,
    video: Res<VideoResource>,
) {
    if video.is_changed() {
        if video.is_selfie_mode {
            maybe_unlock(&mut achievements, Achievement::MirrorMirror);
            if video.is_recording {
                maybe_unlock(&mut achievements, Achievement::FilmStar);
            }
        }
    }
}

fn track_streak_achievements(mut achievements: ResMut<AchievementsState>, streak: Res<Streak>) {
    if streak.is_changed() {
        if streak.current >= 1 {
            maybe_unlock(&mut achievements, Achievement::YouGoGlenCoco);
        }

        if streak.current >= 3 {
            maybe_unlock(&mut achievements, Achievement::InsalataCaprese);
        }
        if streak.current >= 7 {
            maybe_unlock(&mut achievements, Achievement::JelloSalad);
        }
        if streak.current >= 30 {
            maybe_unlock(&mut achievements, Achievement::GreekSalad);
        }
    }
}

fn track_level_completion_achievements(
    mut achievements: ResMut<AchievementsState>,
    found_words: Res<FoundWordsState>,
    current_level: Res<CurrentLevel>,

    sequence_completions: Res<SequenceCompletion>,
    daily_challenge_completions: Res<DailyChallengeCompletion>,
    level_time: Res<LevelTime>,
    daily_challenges: Res<DailyChallenges>,
) {
    if !found_words.is_changed()
        || !found_words.is_level_complete()
        || found_words.word_completions.is_empty()
    {
        return;
    }

    match current_level.as_ref() {
        CurrentLevel::Tutorial { index } => {
            if *index == 1 {
                maybe_unlock(&mut achievements, Achievement::HelloWorld);
            }
        }
        CurrentLevel::Fixed { sequence, .. } => {
            maybe_unlock(&mut achievements, Achievement::ExtraExtra);

            if sequence_completions.get_number_complete_group(&sequence.group()) >= 10 {
                match sequence.group() {
                    ws_levels::level_group::LevelGroup::Geography => {
                        maybe_unlock(&mut achievements, Achievement::KathmanduToKarlsbad);
                    }
                    ws_levels::level_group::LevelGroup::NaturalWorld => {
                        maybe_unlock(&mut achievements, Achievement::LinnaeusCarl);
                    }
                    ws_levels::level_group::LevelGroup::USSports => {
                        //todo maybe have an achievement here
                    },
                }
            }
        }
        CurrentLevel::DailyChallenge { .. } => {
            if daily_challenge_completions.get_next_incomplete_daily_challenge(
                DailyChallenges::get_today_index(),
                &daily_challenges,
            ) == NextDailyChallengeResult::AllFinished
            {
                maybe_unlock(&mut achievements, Achievement::CaesarSalad);
            }
        }
        CurrentLevel::Custom { .. } => {}
        CurrentLevel::NonLevel(_) => {}
    };

    let eligible_for_timed: bool = match current_level.as_ref() {
        CurrentLevel::DailyChallenge { .. } | CurrentLevel::Fixed { .. } => true,
        CurrentLevel::Tutorial { .. } | CurrentLevel::Custom { .. } | CurrentLevel::NonLevel(_) => {
            false
        }
    };

    if !eligible_for_timed {
        return;
    }

    if found_words.hints_used == 0 {
        maybe_unlock(&mut achievements, Achievement::RightInOne);
    }

    let secs = level_time.total_elapsed().as_secs();
    if secs <= 60 {
        maybe_unlock(&mut achievements, Achievement::Pow);
        if secs <= 30 {
            maybe_unlock(&mut achievements, Achievement::Whoosh);
            if secs <= 20 {
                maybe_unlock(&mut achievements, Achievement::Vroom);
            }
        }
    }

    if was_completed_alphabetically(&found_words) {
        maybe_unlock(&mut achievements, Achievement::AlphabetCity);
    } else if was_completed_reverse_alphabetically(&found_words) {
        maybe_unlock(&mut achievements, Achievement::BackToFront);
    }
}

fn was_completed_alphabetically(state: &FoundWordsState) -> bool {
    for (expected, completion) in state.word_completions.iter().enumerate() {
        let Completion::Complete { index } = completion else {
            return false;
        };
        if expected != *index as usize {
            return false;
        }
    }

    true
}

fn was_completed_reverse_alphabetically(state: &FoundWordsState) -> bool {
    for (expected, completion) in state.word_completions.iter().rev().enumerate() {
        let Completion::Complete { index } = completion else {
            return false;
        };
        if expected != *index as usize {
            return false;
        }
    }

    true
}

fn track_found_words(
    mut events: EventReader<WordFoundEvent>,
    mut achievements: ResMut<AchievementsState>,
    mut words_found_count: ResMut<WordsFoundCountState>,
) {
    for WordFoundEvent {
        word, was_hinted, ..
    } in events.read().filter(|x| x.is_first_time)
    {
        words_found_count.count += 1;

        if words_found_count.count == 100 {
            maybe_unlock(&mut achievements, Achievement::TripleDigits);
        } else if words_found_count.count == 1000 {
            maybe_unlock(&mut achievements, Achievement::SamuelJohnson);
        }

        if !was_hinted {
            if word.characters.contains(&Character::Z) {
                maybe_unlock(&mut achievements, Achievement::ZedDeadBaby);
            }
            if word.characters.contains(&Character::X) {
                maybe_unlock(&mut achievements, Achievement::XMarksTheSpot);
            }

            if let Some(length_achievement) = match word.characters.len() {
                8 => Some(Achievement::Octet),
                9 => Some(Achievement::Nonet),
                10 => Some(Achievement::Dectet),
                11 => Some(Achievement::Undectet),
                _ => None,
            } {
                maybe_unlock(&mut achievements, length_achievement);
            }
        }
    }
}

#[derive(Debug, Resource, Clone, PartialEq, Serialize, Deserialize, Default)]
struct AchievementsState {
    pub unlocked: HashSet<Achievement>,
}

impl AchievementsState {
    pub fn resync(&self) {
        for achievement in self.unlocked.iter() {
            Self::unlock_achievement(*achievement);
        }
    }

    /// Unlock the achievement in the connected game service
    pub fn unlock_achievement(achievement: Achievement) {
        info!("Achievement Unlocked: {achievement}");

        #[cfg(any(feature = "android", feature = "ios"))]
        {
            use capacitor_bindings::game_connect::*;
            crate::logging::do_or_report_error(GameConnect::unlock_achievement(
                UnlockAchievementOptions {
                    achievement_id: achievement.android_id().to_string(),
                },
            ));
        }

        #[cfg(feature = "web")]
        {
            spawn_and_run(async move {
                let _ = capacitor_bindings::toast::Toast::show(
                    capacitor_bindings::toast::ShowOptions {
                        text: format!("Achievement Unlocked: {a}", a = achievement.as_ref()),
                        duration: capacitor_bindings::toast::ToastDuration::Long,
                        position: capacitor_bindings::toast::ToastPosition::Top,
                    },
                )
                .await;
            });
        }
    }
}

fn maybe_unlock(state: &mut ResMut<AchievementsState>, achievement: Achievement) {
    if state.bypass_change_detection().unlocked.insert(achievement) {
        state.set_changed();

        AchievementsState::unlock_achievement(achievement);
    }
}

impl TrackableResource for AchievementsState {
    const KEY: &'static str = "achievements";
}

#[derive(Debug, Resource, Clone, PartialEq, Serialize, Deserialize, Default)]
struct WordsFoundCountState {
    pub count: usize,
}

impl TrackableResource for WordsFoundCountState {
    const KEY: &'static str = "WordsFoundCount";
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr, AsRefStr, Display,
)]
#[repr(u8)]
pub enum Achievement {
    /// Alphabet City - solve a puzzle in alphabetical order
    #[strum(serialize = "Alphabet City")]
    AlphabetCity,
    /// Back To Front - solve a puzzle in reverse alphabetical order
    #[strum(serialize = "Back To Front")]
    BackToFront,
    /// Caesar Salad - Complete all available daily puzzles
    #[strum(serialize = "Caesar Salad")]
    CaesarSalad,
    /// Dectet - Find a word that’s 10 letters long
    #[strum(serialize = "Dectet")]
    Dectet,
    /// Extra, Extra: complete a non-daily puzzle
    #[strum(serialize = "Extra, Extra")]
    ExtraExtra,
    /// Film Star - record a video on selfie mode
    #[strum(serialize = "Film Star")]
    FilmStar,
    /// Greek Salad - get a streak of 30 on the daily challenge
    #[strum(serialize = "Greek Salad")]
    GreekSalad,
    /// Hello World - complete the tutorial
    #[strum(serialize = "Hello World")]
    HelloWorld,
    /// Insalata Caprese - get a streak of 3 on the daily challenge
    #[strum(serialize = "Insalata Caprese")]
    InsalataCaprese,
    /// Jello Salad - get a streak of 7 on the daily challenge
    #[strum(serialize = "Jello Salad")]
    JelloSalad,
    /// Kathmandu to Karlsbad - Complete 10 Geography puzzles
    #[strum(serialize = "Kathmandu to Karlsbad")]
    KathmanduToKarlsbad,
    /// Linnaeus, Carl - Complete 10 Natural World Puzzles
    #[strum(serialize = "Linnaeus, Carl")]
    LinnaeusCarl,
    /// Mirror, Mirror - turn on selfie mode
    #[strum(serialize = "Mirror, Mirror")]
    MirrorMirror,
    /// Nonet - find a word that’s 9 letters long
    #[strum(serialize = "Nonet")]
    Nonet,
    /// Octet - find a word that’s 8 letters long
    #[strum(serialize = "Octet")]
    Octet,
    /// Pow! - finish a puzzle in under 1 minute
    #[strum(serialize = "Pow!")]
    Pow,
    /// Q&A  - Use a hint
    #[strum(serialize = "Q&A ")]
    QAndA,
    /// Right in one - complete a puzzle without using a hint
    #[strum(serialize = "Right in one")]
    RightInOne,
    /// Samuel Johnson  - Find 1000 words
    #[strum(serialize = "Samuel Johnson ")]
    SamuelJohnson,
    /// Triple Digits - find 100 words
    #[strum(serialize = "Triple Digits")]
    TripleDigits,
    /// Undectet - find a word that’s 11 letters long
    #[strum(serialize = "Undectet")]
    Undectet,
    /// Vroom! - complete a puzzle in under 20 seconds.
    #[strum(serialize = "Vroom!")]
    Vroom,
    /// Whoosh! - complete a puzzle in under 30 seconds.
    #[strum(serialize = "Whoosh!")]
    Whoosh,
    /// X marks the spot - Find a word that contains an X
    #[strum(serialize = "X marks the spot")]
    XMarksTheSpot,
    /// You go Glen Coco - get a streak of 1
    #[strum(serialize = "You go Glen Coco")]
    YouGoGlenCoco,
    /// Zed's dead, baby - find a word that contains a Z
    #[strum(serialize = "Zed's dead, baby")]
    ZedDeadBaby,
}

impl Achievement {
    pub fn android_id(&self) -> &'static str {
        use Achievement::*;
        //spellchecker:disable
        match self {
            AlphabetCity => "CgkInsjSxL0FEAIQAQ",
            BackToFront => "CgkInsjSxL0FEAIQAg",
            CaesarSalad => "CgkInsjSxL0FEAIQAw",
            Dectet => "CgkInsjSxL0FEAIQBA",
            ExtraExtra => "CgkInsjSxL0FEAIQBQ",
            FilmStar => "CgkInsjSxL0FEAIQBg",
            GreekSalad => "CgkInsjSxL0FEAIQBw",
            HelloWorld => "CgkInsjSxL0FEAIQCA",
            InsalataCaprese => "CgkInsjSxL0FEAIQCQ",
            JelloSalad => "CgkInsjSxL0FEAIQCg",
            KathmanduToKarlsbad => "CgkInsjSxL0FEAIQCw",
            LinnaeusCarl => "CgkInsjSxL0FEAIQDA",
            MirrorMirror => "CgkInsjSxL0FEAIQDQ",
            Nonet => "CgkInsjSxL0FEAIQDg",
            Octet => "CgkInsjSxL0FEAIQDw",
            Pow => "CgkInsjSxL0FEAIQEA",
            QAndA => "CgkInsjSxL0FEAIQEQ",
            RightInOne => "CgkInsjSxL0FEAIQEg",
            SamuelJohnson => "CgkInsjSxL0FEAIQEw",
            TripleDigits => "CgkInsjSxL0FEAIQFA",
            Undectet => "CgkInsjSxL0FEAIQFQ",
            Vroom => "CgkInsjSxL0FEAIQFg",
            Whoosh => "CgkInsjSxL0FEAIQFw",
            XMarksTheSpot => "CgkInsjSxL0FEAIQGA",
            YouGoGlenCoco => "CgkInsjSxL0FEAIQGQ",
            ZedDeadBaby => "CgkInsjSxL0FEAIQGg",
        }
        //spellchecker:enable
    }
}
