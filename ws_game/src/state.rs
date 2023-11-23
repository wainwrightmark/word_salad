use std::num::NonZeroUsize;

use crate::{
    completion::{track_level_completion, TotalCompletion},
    prelude::*,
};
use itertools::Itertools;
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::EnumIs;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_resource::<MenuState>();
        app.init_tracked_resource::<CurrentLevel>();
        app.init_tracked_resource::<FoundWordsState>();
        app.init_tracked_resource::<TotalCompletion>();

        app.add_systems(Update, track_found_words);
        app.add_systems(Update, track_level_completion);
        // app.add_systems(Update, track_level_change);
    }
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChosenState {
    pub solution: Solution,
    pub is_just_finished: bool,
}

impl ChosenState {
    const EMPTY_SOLUTION: &'static Solution = &Solution::new_const();
    pub fn current_solution(&self) -> &Solution {
        if self.is_just_finished {
            Self::EMPTY_SOLUTION
        } else {
            &self.solution
        }
    }
}

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct FoundWordsState {
    pub unneeded_tiles: GridSet,
    pub word_completions: Vec<Completion>,
    pub hints_used: usize,
}

impl Default for FoundWordsState {
    fn default() -> Self {
        Self::new_from_level(&CurrentLevel::default())
    }
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWords";
}

impl FoundWordsState {
    pub fn new_from_level(current_level: &CurrentLevel) -> Self {
        let level = current_level.level();
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

    pub fn auto_hint_set(&self, level: &DesignedLevel, solution: &Solution) -> GridSet {
        self.hint_set::<false>(level, solution)
    }

    fn hint_set<const MANUAL: bool>(
        &self,
        level: &DesignedLevel,
        solution: &Solution,
    ) -> GridSet {
        let mut set = GridSet::default();
        let adjusted_grid = self.adjusted_grid(level);



        if solution.is_empty() {
            //hint all known first letters
            for (word, completion) in level.words.iter().zip(self.word_completions.iter()) {
                if !(MANUAL && completion.is_manual_hinted()
                    || (!MANUAL && completion.is_auto_hinted()))
                {
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
                    (Completion::AutoHinted(hints), false) => hints,
                    (Completion::ManualHinted(hints), true) => hints,
                    _ => {
                        continue;
                    }
                };

                if hints.get() <= solution.len() {
                    continue;
                };

                if let Some(solution) = word.find_solution(&adjusted_grid) {
                    if solution.starts_with(solution.as_slice()) {
                        if let Some(tile) = solution.get(solution.len()) {
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

    pub fn get_completion(&self, word_index: usize) -> Completion {
        self.word_completions
            .get(word_index)
            .unwrap_or(&Completion::Complete)
            .clone()
    }

    pub fn try_hint_word(&mut self, current_level: &CurrentLevel, word_index: usize) -> bool {
        let level = current_level.level();

        let Some(completion) = self.word_completions.get_mut(word_index) else {
            return false;
        };
        let Some(word) = level.words.get(word_index) else {
            return false;
        };

        match completion {
            Completion::Unstarted => {
                *completion = Completion::ManualHinted(NonZeroUsize::MIN);
                self.hints_used += 1;
            }
            Completion::ManualHinted(hints) => {
                if hints.get() >= word.characters.len() {
                    return false;
                }
                *hints = hints.saturating_add(1);
                self.hints_used += 1;
            }
            Completion::Complete => return false,
            Completion::AutoHinted(hints) => {
                if hints.get() >= word.characters.len() {
                    return false;
                }
                let new_hints = hints.saturating_add(1);
                *hints = new_hints;
                *completion = Completion::ManualHinted(new_hints);
            }
        }
        return true;
    }

    pub fn try_hint(&mut self, current_level: &CurrentLevel) -> bool {
        let level = current_level.level();

        let mut min_hints = usize::MAX;
        let mut min_hint_index: Option<usize> = None;

        'check: for (index, (word, completion)) in level
            .words
            .iter()
            .zip(self.word_completions.iter())
            .enumerate()
        {
            match completion {
                Completion::Unstarted => {
                    min_hints = 0;
                    min_hint_index = Some(index);
                    break 'check;
                }
                Completion::ManualHinted(hints) | Completion::AutoHinted(hints) => {
                    if hints.get() < word.characters.len() && hints.get() < min_hints {
                        min_hints = hints.get();
                        min_hint_index = Some(index)
                    }
                }
                Completion::Complete => {}
            }
        }

        let Some(index) = min_hint_index else {
            return false;
        };
        self.word_completions[index] =
            Completion::ManualHinted(NonZeroUsize::MIN.saturating_add(min_hints));

        true
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize, EnumIs)]
pub enum Completion {
    Unstarted,
    AutoHinted(NonZeroUsize),
    ManualHinted(NonZeroUsize),
    Complete,
}

impl Completion {
    pub fn color(&self) -> &'static Color {
        const UNSTARTED: &'static Color = &convert_color(palette::WORD_BACKGROUND_UNSTARTED);
        const MANUAL: &'static Color = &convert_color(palette::WORD_BACKGROUND_MANUAL_HINT);
        const COMPLETE: &'static Color = &convert_color(palette::WORD_BACKGROUND_COMPLETE);
        const AUTO: &'static Color = &convert_color(palette::WORD_BACKGROUND_AUTO_HINT);

        match self {
            Completion::Unstarted => UNSTARTED,
            Completion::ManualHinted(_) => MANUAL,
            Completion::Complete => COMPLETE,
            Completion::AutoHinted(_) => AUTO,
        }
    }

    pub fn known_characters<'a, 'w>(&'a self, word: &'w DisplayWord) -> Option<&'w [Character]> {
        match self {
            Completion::Unstarted => None,
            Completion::Complete => Some(&word.characters),
            Completion::AutoHinted(hints) | Completion::ManualHinted(hints) => Some(
                &word
                    .characters
                    .split_at(hints.get().min(word.characters.len()))
                    .0,
            ),
        }
    }
}

fn track_found_words(
    mut commands: Commands,
    mut chosen: ResMut<ChosenState>,
    current_level: Res<CurrentLevel>,
    mut found_words: ResMut<FoundWordsState>,
    asset_server: Res<AssetServer>,
    size: Res<Size>,
) {
    if !chosen.is_changed() || chosen.is_just_finished {
        return;
    }
    let grid = current_level.level().grid;
    let chars: CharsArray = chosen.solution.iter().map(|t| grid[*t]).collect();

    let level = current_level.level();
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

    crate::animated_solutions::animate_solution(
        &mut commands,
        &chosen.solution,
        word,
        is_first_time,
        &asset_server,
        &size,
        &current_level,
    );

    found_words.calculate_auto_hints(&current_level);

    if is_first_time {
        chosen.is_just_finished = true;
    }
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
            .map(|(completion, word)| match completion {
                Completion::Unstarted => None, //a `None` is a word that we can check
                Completion::AutoHinted(h) | Completion::ManualHinted(h) => {
                    Some(&word.characters.as_slice()[..h.get()])
                }
                Completion::Complete => Some(word.characters.as_slice()),
            })
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

            if let Some(slice) = slices.peek() {
                if let Some(slice) = slice {
                    if !could_precede(&chosen_characters, slice) {
                        continue;
                    }
                    if slice.starts_with(&chosen_characters) {
                        successor = slice.iter().skip(chosen_characters.len()).cloned().next();
                    }
                }
            }
            if predecessor.is_none() && successor.is_none() {
                //info!("No pre or succ");
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

    fn calculate_auto_hints(&mut self, level: &CurrentLevel) {
        let level = level.level();

        let mut word_index = 0;

        while let Some(completion) = self.word_completions.get(word_index) {
            if completion.is_manual_hinted() || completion.is_complete() {
                word_index += 1;
                continue;
            }

            let preceder: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .take(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .rev()
                .next()
                .unwrap_or_default();

            let successor: &[Character] = self
                .word_completions
                .iter()
                .zip(level.words.iter())
                .skip(word_index)
                .flat_map(|(c, w)| c.known_characters(w))
                .next()
                .unwrap_or_default();

            if !(preceder.is_empty() && successor.is_empty()) {
                if let Some(letters) = level
                    .words
                    .get(word_index)
                    .and_then(|x| NonZeroUsize::new(x.characters.len()))
                {
                    let hints = self
                        .unneeded_tiles
                        .negate()
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
                            self.word_completions[word_index] = Completion::AutoHinted(hints);
                        }
                        Err(_) => {}
                    }
                }
            }

            word_index += 1;
        }
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
/// Otherwise return one
fn count_hints(
    tile: Tile,
    grid: &Grid,
    unneeded_tiles: GridSet,
    preceder: &[Character],
    successor: &[Character],
    remaining_letters: NonZeroUsize,
) -> Option<std::num::NonZeroUsize> {
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
        .and_then(|x| NonZeroUsize::new(x))
    {
        Some(r) => r,
        None => {
            return Some(NonZeroUsize::MIN);
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
                return None;
            } else {
                //multiple children so this is as far as it goes
                Some(NonZeroUsize::MIN)
            }
        }
    }
}

#[cfg(test)]
pub mod tests {

    use std::num::NonZeroUsize;

    use crate::prelude::{Completion, CurrentLevel, DesignedLevel, FoundWordsState};

    #[test]
    pub fn test_auto_hints() {
        //TODO test the following with everything but croatia
        // spellchecker:disable-next-line
        //PLTAOAYIMRNDFCEG	Europe Countries 6	Croatia 	France  	Germany 	Italy   	Malta   	Poland  	Romania

        let level = DesignedLevel::from_tsv_line(
            // spellchecker:disable-next-line
            "DNGLHUAOSTRPAIYC	Europe Countries 2	Austria 	Croatia 	Cyprus  	Hungary 	Poland  	Portugal",
        )
        .unwrap();
        let cl = CurrentLevel::Custom(level);
        let mut found_words = FoundWordsState::new_from_level(&cl);

        found_words.calculate_auto_hints(&cl);

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

        found_words.update_unneeded_tiles(cl.level());

        found_words.calculate_auto_hints(&cl);

        let croatia_completion = found_words.word_completions[1];

        assert_eq!(
            croatia_completion,
            Completion::AutoHinted(NonZeroUsize::new(2).unwrap())
        );
    }
}
