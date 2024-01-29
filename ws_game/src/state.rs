use std::{num::NonZeroUsize, time::Duration};

use crate::{completion::*, prelude::*};
use itertools::{Either, Itertools};
use nice_bevy_utils::{CanInitTrackedResource, CanRegisterAsyncEvent, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use ws_levels::level_sequence::LevelSequence;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_resource::<MenuState>();
        app.init_tracked_resource::<CurrentLevel>();
        app.init_tracked_resource::<FoundWordsState>();
        app.init_tracked_resource::<SequenceCompletion>();
        app.init_tracked_resource::<DailyChallengeCompletion>();
        app.init_tracked_resource::<TutorialCompletion>();
        app.init_tracked_resource::<HintState>();
        app.init_tracked_resource::<SavedLevelsState>();

        app.add_event::<AnimateSolutionsEvent>();
        app.register_async_event::<ChangeLevelEvent>();

        app.add_systems(Update, track_found_words);
        app.add_systems(Update, track_level_completion);

        app.add_systems(Update, animate_solutions.after(track_found_words));
        app.add_systems(Update, remove_animated_solutions_on_complete.run_if(|f: Res<FoundWordsState>| f.is_changed() && f.is_level_complete()));

        app.add_systems(
            PostUpdate,
            handle_change_level_event.run_if(|ev: EventReader<ChangeLevelEvent>| !ev.is_empty()),
        );
    }
}
#[derive(Debug, Event)]
pub enum ChangeLevelEvent {
    ChangeTo(CurrentLevel),
    Reset,
}

impl From<CurrentLevel> for ChangeLevelEvent {
    fn from(value: CurrentLevel) -> Self {
        Self::ChangeTo(value)
    }
}

fn handle_change_level_event(
    mut events: EventReader<ChangeLevelEvent>,
    mut current_level: ResMut<CurrentLevel>,
    daily_challenges: Res<DailyChallenges>,
    mut found_words: ResMut<FoundWordsState>,
    mut chosen: ResMut<ChosenState>,
    mut saved_levels: ResMut<SavedLevelsState>,

    mut time: ResMut<LevelTime>,
) {
    for event in events.read() {
        let new_level = match event {
            ChangeLevelEvent::ChangeTo(new_level) => {
                if new_level == current_level.as_ref() {
                    continue;
                }
                new_level
            }
            ChangeLevelEvent::Reset => {
                if let Either::Left(level) = current_level.level(daily_challenges.as_ref()) {
                    *time = LevelTime::default();
                    *found_words = FoundWordsState::new_from_level(level);
                }

                return;
            }
        };

        let previous_key = SavedLevelKey::try_from_current(&current_level);
        let new_key = SavedLevelKey::try_from_current(&new_level);

        if let Some(previous_key) = previous_key {
            if found_words.is_level_started() {
                if !found_words.is_level_complete() {
                    let state = found_words.clone();
                    let elapsed = time.total_elapsed();
                    let saved_state = SavedState {
                        found_words_state: state,
                        elapsed,
                    };
                    saved_levels.insert(previous_key, saved_state);
                }
            }
        }

        let loaded_level = new_key
            .and_then(|k| saved_levels.remove(k))
            .filter(|_| previous_key != new_key);

        let saved_state = match new_level.level(daily_challenges.as_ref()) {
            Either::Left(level) => loaded_level.unwrap_or_else(|| SavedState {
                elapsed: Duration::ZERO,
                found_words_state: FoundWordsState::new_from_level(level),
            }),
            Either::Right(..) => SavedState {
                elapsed: Duration::ZERO,
                found_words_state: FoundWordsState::default(),
            },
        };

        *current_level = new_level.clone();
        *found_words = saved_state.found_words_state;
        *time = LevelTime::Paused {
            elapsed: saved_state.elapsed,
        };
        time.resume_timer();

        *chosen = ChosenState::default();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedLevelKey {
    DailyChallenge {
        index: usize,
    },
    Sequence {
        level_index: usize,
        sequence: LevelSequence,
    },
}

impl SavedLevelKey {
    pub fn try_from_current(current_level: &CurrentLevel) -> Option<Self> {
        match current_level {
            CurrentLevel::Tutorial { .. } => None,
            CurrentLevel::Fixed {
                level_index,
                sequence,
            } => Some(SavedLevelKey::Sequence {
                level_index: *level_index,
                sequence: *sequence,
            }),
            CurrentLevel::DailyChallenge { index } => {
                Some(SavedLevelKey::DailyChallenge { index: *index })
            }
            CurrentLevel::Custom { .. } => None,
            CurrentLevel::NonLevel(_) => None,
        }
    }
}

#[derive(Debug, Resource, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct SavedLevelsState {
    saved_sequences: std::collections::BTreeMap<u64, SavedState>, //keyed by (sequence * u32::max, index, )
    saved_daily: std::collections::BTreeMap<u32, SavedState>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct SavedState {
    elapsed: Duration,
    found_words_state: FoundWordsState,
}

impl SavedLevelsState {
    fn insert(&mut self, key: SavedLevelKey, value: SavedState) -> Option<SavedState> {
        match key {
            SavedLevelKey::DailyChallenge { index } => self.saved_daily.insert(index as u32, value),
            SavedLevelKey::Sequence {
                level_index,
                sequence,
            } => self.saved_sequences.insert(
                ((sequence as u64) * (u32::MAX as u64)) + level_index as u64,
                value,
            ),
        }
    }

    fn remove(&mut self, key: SavedLevelKey) -> Option<SavedState> {
        match key {
            SavedLevelKey::DailyChallenge { index } => self.saved_daily.remove(&(index as u32)),
            SavedLevelKey::Sequence {
                level_index,
                sequence,
            } => self
                .saved_sequences
                .remove(&(((sequence as u64) * (u32::MAX as u64)) + level_index as u64)),
        }
    }
}

impl TrackableResource for SavedLevelsState {
    const KEY: &'static str = "SavedLevelsState";
}

const INITIAL_HINTS: usize = 3;

#[derive(Debug, Clone, Resource, Serialize, Deserialize, MavericContext, PartialEq)]
pub struct HintState {
    pub hints_remaining: usize,
    pub total_earned_hints: usize,
    pub total_bought_hints: usize,
}

impl Default for HintState {
    fn default() -> Self {
        Self {
            hints_remaining: INITIAL_HINTS,
            total_earned_hints: 0,
            total_bought_hints: 0,
        }
    }
}

impl TrackableResource for HintState {
    const KEY: &'static str = "HintState";
}

#[derive(
    Debug, Clone, Resource, Serialize, Deserialize, MavericContext, Default, PartialEq, Eq,
)]
pub struct FoundWordsState {
    pub unneeded_tiles: GridSet,
    pub word_completions: Vec<Completion>,
    pub hints_used: usize,
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWords";
}

impl FoundWordsState {
    pub fn new_from_level(level: &DesignedLevel) -> Self {
        Self {
            unneeded_tiles: GridSet::EMPTY,
            word_completions: vec![Completion::Unstarted; level.words.len()],
            hints_used: 0,
        }
    }

    /// Grid with unneeded characters blanked
    fn adjusted_grid(&self, level: &DesignedLevel) -> Grid {
        let mut grid = level.grid;

        for tile in self.unneeded_tiles.iter_true_tiles() {
            grid[tile] = Character::Blank;
        }

        grid
    }

    pub fn manual_hint_set(&self, level: &DesignedLevel, solution: &Solution) -> GridSet {
        self.hint_set::<true>(level, solution)
    }

    fn hint_set<const MANUAL: bool>(&self, level: &DesignedLevel, solution: &Solution) -> GridSet {
        let mut set = GridSet::default();
        let adjusted_grid = self.adjusted_grid(level);

        if solution.is_empty() {
            //hint all known first letters
            for (word, completion) in level.words.iter().zip(self.word_completions.iter()) {
                if !(
                    MANUAL && completion.is_manual_hinted()
                    // || (!MANUAL && completion.is_auto_hinted())
                ) {
                    continue;
                }

                if let Some(solution) = word.find_solution(&adjusted_grid) {
                    if let Some(first) = solution.first() {
                        set.set_bit(first, true)
                    }
                }
            }
        } else {
            // hint all solutions starting with this
            for (word, completion) in level.words.iter().zip(self.word_completions.iter()) {
                let hints = match (completion, MANUAL) {
                    (Completion::ManualHinted(hints), true) => hints,
                    _ => {
                        continue;
                    }
                };

                if let Some(word_solution) = word.find_solution(&adjusted_grid) {
                    let len = hints.get().min(solution.len());

                    if solution.iter().take(len).eq(word_solution.iter().take(len)) {
                        for tile in word_solution.iter().take(hints.get()) {
                            set.set_bit(tile, true)
                        }
                    }
                }
            }
        }
        set
    }

    pub fn is_level_complete(&self) -> bool {
        self.word_completions.iter().all(|x| x.is_complete())
    }

    pub fn is_level_started(&self) -> bool {
        self.word_completions.iter().any(|x| !x.is_unstarted())
    }

    pub fn get_completion(&self, word_index: usize) -> Completion {
        *self
            .word_completions
            .get(word_index)
            .unwrap_or(&Completion::Complete)
    }

    pub fn try_hint_word(
        &mut self,
        hint_state: &mut HintState,
        level: &DesignedLevel,
        word_index: usize,
        chosen_state: &mut ChosenState,
        ew: &mut impl AnyEventWriter<AnimateSolutionsEvent>,
    ) -> bool {
        let Some(new_hints) = hint_state.hints_remaining.checked_sub(1) else {
            return false;
        };

        let Some(word) = level.words.get(word_index) else {
            return false;
        };

        let min_hint_count = NonZeroUsize::MIN.saturating_add(self.count_selected_characters(
            level,
            word_index,
            chosen_state,
        ));

        let Some(completion) = self.word_completions.get_mut(word_index) else {
            return false;
        };

        let new_count = match completion {
            Completion::Unstarted => {
                *completion = Completion::ManualHinted(min_hint_count);
                self.hints_used += 1;
                hint_state.hints_remaining = new_hints;

                min_hint_count.get()
            }
            Completion::ManualHinted(hints) => {
                if hints.get() >= word.characters.len() {
                    return false;
                }
                *hints = hints.saturating_add(1);

                if min_hint_count > *hints {
                    *hints = min_hint_count;
                }

                self.hints_used += 1;
                hint_state.hints_remaining = new_hints;

                hints.get()
            }
            Completion::Complete => return false,
        };

        if let Some(solution) = word.find_solution_with_tiles(&level.grid, self.unneeded_tiles) {
            let new_selection: ArrayVec<Tile, 16> =
                ArrayVec::from_iter(solution.iter().take(new_count).cloned());
            chosen_state.solution = new_selection;

            if solution.len() > new_count {
                chosen_state.is_just_finished = false;
            } else {
                //do not select the full word - let the user do that
                *completion = Completion::Complete;
                self.update_unneeded_tiles(level);

                ew.send(AnimateSolutionsEvent {
                    solution,
                    is_first_time: true,
                    word: word.clone(),
                    level: level.clone(),
                });

                chosen_state.is_just_finished = true; //todo change this slightly
            }
        } else {
            warn!("Could not find solution during hint");
        }

        true
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize, EnumIs, Default)]
pub enum Completion {
    #[default]
    Unstarted,
    // AutoHinted(NonZeroUsize),
    ManualHinted(NonZeroUsize),
    Complete,
}

impl Completion {
    pub fn color(&self) -> &'static Color {
        const UNSTARTED: &Color = &convert_color_const(palette::WORD_BACKGROUND_UNSTARTED);
        const MANUAL: &Color = &convert_color_const(palette::WORD_BACKGROUND_MANUAL_HINT);
        const COMPLETE: &Color = &convert_color_const(palette::WORD_BACKGROUND_COMPLETE);

        match self {
            Completion::Unstarted => UNSTARTED,
            Completion::ManualHinted(_) => MANUAL,
            Completion::Complete => COMPLETE,
        }
    }

    pub fn known_characters<'w>(&self, word: &'w DisplayWord) -> Option<&'w [Character]> {
        match self {
            Completion::Unstarted => None,
            Completion::Complete => Some(&word.characters),
            Completion::ManualHinted(hints) => Some(
                word.characters
                    .split_at(hints.get().min(word.characters.len()))
                    .0,
            ),
        }
    }
}

fn track_found_words(
    mut chosen: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    daily_challenges: Res<DailyChallenges>,
    mut ew: EventWriter<AnimateSolutionsEvent>,
) {
    if !chosen.is_changed() || chosen.is_just_finished {
        return;
    }
    let Either::Left(level) = current_level.level(&daily_challenges) else {
        return;
    };
    let grid = level.grid;
    let chars: CharsArray = chosen.solution.iter().map(|t| grid[*t]).collect();

    let Some((word_index, word)) = level
        .words
        .iter()
        .enumerate()
        .find(|(_, word)| word.characters == chars)
    else {
        return;
    };

    let Some(completion) = found_words.word_completions.get(word_index) else {
        return;
    };

    let is_first_time = !completion.is_complete();
    if is_first_time {
        found_words.word_completions[word_index] = Completion::Complete;

        found_words.update_unneeded_tiles(level);
    }

    ew.send(AnimateSolutionsEvent {
        solution: chosen.solution.clone(),
        is_first_time,
        word: word.clone(),
        level: level.clone(),
    });

    if is_first_time {
        chosen.is_just_finished = true;
    }

    //CODE FOR GIVING FREE HINTS DURING TUTORIAL
    // if is_first_time
    //     && current_level
    //         .as_ref()
    //         .eq(&CurrentLevel::Tutorial { index: 1 })
    // {
    //     let complete_words = found_words
    //         .word_completions
    //         .iter()
    //         .filter(|x| x.is_complete())
    //         .count();

    //     if let Some(hints_to_give) = match complete_words {
    //         1 => NonZeroUsize::new(1),
    //         4 => NonZeroUsize::new(2),
    //         _ => None,
    //     } {
    //         if let Some((word_index, _)) = found_words
    //             .word_completions
    //             .iter()
    //             .find_position(|x| !x.is_complete())
    //         {
    //             found_words.word_completions[word_index] =
    //                 Completion::ManualHinted(hints_to_give);

    //             if let Some(level)  = get_tutorial_level(1){
    //                 if let Some(word) = level.words.get(word_index){
    //                     if let Some(mut solution) = word.find_solution_with_tiles(&level.grid, found_words.unneeded_tiles){
    //                         solution.truncate(hints_to_give.get());
    //                         chosen.solution = solution;
    //                         chosen.is_just_finished = false;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}

impl FoundWordsState {
    fn update_unneeded_tiles(&mut self, level: &DesignedLevel) {
        self.unneeded_tiles = level.calculate_unneeded_tiles(self.unneeded_tiles, |index| {
            self.word_completions
                .get(index)
                .map(|x| x.is_complete())
                .unwrap_or(true)
        });
    }

    /// Inadvisable tiles are tiles that are selectable, but can't lead to a solution
    pub fn calculate_inadvisable_tiles(
        &self,
        current_solution: &Solution,
        level: &DesignedLevel,
    ) -> GridSet {
        let mut selectable = match current_solution.last() {
            Some(tile) => GridSet::from_iter(tile.iter_adjacent()),
            None => GridSet::ALL,
        };

        for tile in current_solution {
            selectable.set_bit(tile, false);
        }

        let mut inadvisable = selectable.intersect(&self.unneeded_tiles.negate());

        let chosen_characters: ArrayVec<Character, 16> =
            current_solution.iter().map(|x| level.grid[*x]).collect();

        let mut slices = self
            .word_completions
            .iter()
            .zip(level.words.iter())
            .map(|(completion, word)| completion.known_characters(word))
            //dedup to remove consecutive `None`
            .dedup()
            .peekable();

        let mut predecessor: Option<Character> = None;
        //let mut count = 0;

        while let Some(slice) = slices.next() {
            //   count += 1;
            //todo check length of this word
            if let Some(slice) = slice {
                if !could_precede(slice, &chosen_characters) {
                    //info!("Went past prefix after {count}");
                    return inadvisable;
                }

                if slice.starts_with(&chosen_characters) {
                    predecessor = slice.iter().skip(chosen_characters.len()).cloned().next();
                }
                continue;
            }

            let mut successor: Option<Character> = None;

            if let Some(Some(slice)) = slices.peek() {
                if !could_precede(&chosen_characters, slice) {
                    continue;
                }
                if slice.starts_with(&chosen_characters) {
                    successor = slice.iter().skip(chosen_characters.len()).cloned().next();
                }
            }
            if predecessor.is_none() && successor.is_none() {
                //info!("No pre or successor");
                return GridSet::EMPTY;
            }

            'tiles: for tile in inadvisable.clone().iter_true_tiles() {
                let character = level.grid[tile];
                if let Some(p) = predecessor {
                    if p.as_char() > character.as_char() {
                        continue 'tiles;
                    }
                }
                if let Some(s) = successor {
                    if s.as_char() < character.as_char() {
                        continue 'tiles;
                    }
                }
                inadvisable.set_bit(&tile, false);
            }
            if inadvisable.is_empty() {
                //info!("All bits unset");
                return GridSet::EMPTY;
            }
        }
        //info!("Checked all words {count}");
        inadvisable
    }

    fn count_selected_characters(
        &self,
        level: &DesignedLevel,
        word_index: usize,
        chosen: &ChosenState,
    ) -> usize {
        if chosen.solution.is_empty() || chosen.is_just_finished {
            return 0;
        }

        let Some(word) = level.words.get(word_index) else {
            return 0;
        };

        chosen
            .solution
            .iter()
            .zip(word.characters.iter())
            .take_while(|(tile, character)| level.grid[**tile] == **character)
            .count()
    }

    #[allow(dead_code)]
    fn count_inevitable_characters(&self, level: &DesignedLevel, word_index: usize) -> usize {
        if let Some(completion) = self.word_completions.get(word_index) {
            let prefix_characters = match completion {
                Completion::Unstarted => 0,
                Completion::ManualHinted(a) => a.get(),
                Completion::Complete => return 0,
            };

            let preceder: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .take(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .next_back()
                .unwrap_or_default();

            let successor: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .skip(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .next()
                .unwrap_or_default();

            if let Some(letters) = level
                .words
                .get(word_index)
                .and_then(|x| NonZeroUsize::new(x.characters.len()))
            {
                let (initial_tiles, preceder, successor) = if prefix_characters == 0 {
                    (self.unneeded_tiles.negate(), preceder, successor)
                } else {
                    let preceder = if prefix_characters > preceder.len() {
                        &[]
                    } else {
                        preceder.split_at(prefix_characters).1
                    };
                    let successor = if prefix_characters > successor.len() {
                        &[]
                    } else {
                        successor.split_at(prefix_characters).1
                    };

                    let Some(w) = level.words.get(word_index) else {
                        return 0;
                    };
                    let Some(solution) = w.find_solution(&level.grid) else {
                        return 0;
                    };

                    let Some(tile) = solution.get(prefix_characters.saturating_sub(1)) else {
                        return 0;
                    };

                    (
                        GridSet::from_iter(tile.iter_adjacent())
                            .intersect(&self.unneeded_tiles.negate()),
                        preceder,
                        successor,
                    )
                };

                let hints = initial_tiles
                    .iter_true_tiles()
                    .flat_map(|tile| {
                        count_hints(
                            tile,
                            &level.grid,
                            self.unneeded_tiles,
                            preceder,
                            successor,
                            letters,
                        )
                    })
                    .exactly_one();

                match hints {
                    Ok(hints) => {
                        return hints;
                    }
                    Err(_) => {}
                }
            }
        }

        0
    }
}

fn could_precede(p: &[Character], s: &[Character]) -> bool {
    for (p, s) in p.iter().zip(s.iter()) {
        match p.as_char().cmp(&s.as_char()) {
            std::cmp::Ordering::Less => return true,
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => return false,
        }
    }

    true
}

/// If this doesn't come between the preceder and succeeder, return None
/// If there is exactly one child, which returns a value greater than zero, return that value + 1
/// Otherwise return zero
#[allow(dead_code)]
fn count_hints(
    tile: Tile,
    grid: &Grid,
    unneeded_tiles: GridSet,
    preceder: &[Character],
    successor: &[Character],
    remaining_letters: NonZeroUsize,
) -> Option<usize> {
    let character = grid[tile];

    let next_preceder = match preceder.split_first() {
        Some((c, next)) => match character.as_char().cmp(&c.as_char()) {
            std::cmp::Ordering::Less => return None,
            std::cmp::Ordering::Equal => next,
            std::cmp::Ordering::Greater => &[],
        },
        None => &[],
    };

    let next_succeeder = match successor.split_first() {
        Some((c, next)) => match character.as_char().cmp(&c.as_char()) {
            std::cmp::Ordering::Less => &[],
            std::cmp::Ordering::Equal => next,
            std::cmp::Ordering::Greater => return None,
        },
        None => &[],
    };

    let next_unneeded = unneeded_tiles.with_bit_set(&tile, true);
    let next_remaining_letters = match remaining_letters
        .get()
        .checked_sub(1)
        .and_then(NonZeroUsize::new)
    {
        Some(r) => r,
        None => {
            return Some(1);
        }
    };

    let child = tile
        .iter_adjacent()
        .filter(|next| !unneeded_tiles.get_bit(next))
        .flat_map(|next_tile| {
            count_hints(
                next_tile,
                grid,
                next_unneeded,
                next_preceder,
                next_succeeder,
                next_remaining_letters,
            )
        })
        .exactly_one();

    match child {
        Ok(count) => Some(count.saturating_add(1)),
        Err(error) => {
            if error.count() == 0 {
                //no possible children
                None
            } else {
                //multiple children so this is as far as it goes
                Some(1)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::num::NonZeroUsize;

    use crate::{
        chosen_state::ChosenState,
        prelude::{Completion, DesignedLevel, FoundWordsState, TestEventWriter},
        state::HintState,
    };

    #[test]
    pub fn test_inevitable_characters() {
        //TODO test the following with everything but croatia
        // spellchecker:disable-next-line
        //PLTAOAYIMRNDFCEG	Europe Countries 6	Croatia 	France  	Germany 	Italy   	Malta   	Poland  	Romania

        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "DNGLHUAOSTRPAIYC	Europe Countries 2	Austria 	Croatia 	Cyprus  	Hungary 	Poland  	Portugal",
        )
        .unwrap();

        let mut found_words = FoundWordsState::new_from_level(&level);

        for index in 0..found_words.word_completions.len() {
            let characters = found_words.count_inevitable_characters(&level, index);

            assert_eq!(characters, 0);
        }

        for word in found_words.word_completions.iter() {
            assert!(
                word.is_unstarted(),
                "All words should be unstarted at this stage"
            )
        }

        for (index, completion) in found_words.word_completions.iter_mut().enumerate() {
            if index != 1 {
                *completion = Completion::Complete;
            }
        }

        found_words.update_unneeded_tiles(&level);

        let c = found_words.count_inevitable_characters(&level, 1);

        assert_eq!(c, 2);
    }

    #[test]
    pub fn test_auto_hints2() {
        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "SWEDLVNEOMAI_RKA	5	Denmark 	Romania 	Slovakia	Slovenia	Sweden",
        )
        .unwrap();

        let mut found_words = FoundWordsState::new_from_level(&level);
        let mut hint_state = HintState {
            hints_remaining: 10,
            total_earned_hints: 0,
            total_bought_hints: 0,
        };
        let mut chosen_state = ChosenState::default();

        let mut event_writer = TestEventWriter::default();

        let hinted = found_words.try_hint_word(
            &mut hint_state,
            &level,
            4,
            &mut chosen_state,
            &mut event_writer,
        );

        assert!(hinted);
        assert_eq!(hint_state.hints_remaining, 9);

        assert_eq!(
            found_words.word_completions.get(4).unwrap(),
            &Completion::ManualHinted(NonZeroUsize::new(1).unwrap())
        );
    }
}
