use std::num::NonZeroUsize;

use crate::prelude::*;
use itertools::Itertools;
use nice_bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};
use strum::EnumIs;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChosenState>();
        app.init_tracked_resource::<CurrentLevel>();
        app.init_tracked_resource::<FoundWordsState>();

        app.add_systems(Update, track_found_words);
        // app.add_systems(Update, track_level_change);
    }
}

#[derive(Debug, Clone, Resource, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChosenState(pub Solution);

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
    fn adjusted_grid(&self, level: &CurrentLevel) -> Grid {
        let mut grid = level.level().grid;

        for tile in self.unneeded_tiles.iter_true_tiles() {
            grid[tile] = Character::Blank;
        }

        grid
    }

    pub fn hint_set(&self, level: &CurrentLevel, chosen_state: &ChosenState) -> GridSet {
        let mut set = GridSet::default();
        let adjusted_grid = self.adjusted_grid(level);

        if chosen_state.0.is_empty() {
            //hint all known first letters
            for (word, completion) in level.level().words.iter().zip(self.word_completions.iter()) {
                let Completion::Hinted(..) = completion else {
                    continue;
                };

                if let Some(solution) = word.find_solution(&adjusted_grid) {
                    if let Some(first) = solution.first() {
                        set.set_bit(first, true)
                    }
                }
            }
        } else {
            // hint all solutions starting with this
            for (word, completion) in level.level().words.iter().zip(self.word_completions.iter()) {
                let Completion::Hinted(hints) = completion else {
                    continue;
                };

                if hints <= &chosen_state.0.len() {
                    continue;
                };

                if let Some(solution) = word.find_solution(&adjusted_grid) {
                    if solution.starts_with(chosen_state.0.as_slice()) {
                        if let Some(tile) = solution.get(chosen_state.0.len()) {
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
                *completion = Completion::Hinted(1);
                self.hints_used += 1;
            }
            Completion::Hinted(hints) => {
                if *hints >= word.characters.len() {
                    return false;
                }
                *hints = *hints + 1;
                self.hints_used += 1;
            }
            Completion::Complete => return false,
            Completion::AutoHinted(hints) => {
                if *hints >= word.characters.len() {
                    return false;
                }
                *hints = *hints + 1;
                *completion = Completion::Hinted(*hints + 1);
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
                Completion::Hinted(hints) | Completion::AutoHinted(hints) => {
                    if *hints < word.characters.len() && *hints < min_hints {
                        min_hints = *hints;
                        min_hint_index = Some(index)
                    }
                }
                Completion::Complete => {}
            }
        }

        let Some(index) = min_hint_index else {
            return false;
        };
        self.word_completions[index] = Completion::Hinted(min_hints + 1);

        true
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize, EnumIs)]
pub enum Completion {
    Unstarted,
    AutoHinted(usize),
    Hinted(usize),
    Complete,
}

impl Completion {
    pub fn color(&self) -> &'static Color {
        const INCOMPLETE_COLOR: Color = Color::ALICE_BLUE;
        const HINT_COLOR: Color = Color::rgb(0.3, 0.3, 0.9);
        const AUTO_HINT_COLOR: Color = Color::SILVER;
        const COMPLETE_COLOR: Color = Color::GREEN;

        match self {
            Completion::Unstarted => &INCOMPLETE_COLOR,
            Completion::Hinted(_) => &HINT_COLOR,
            Completion::Complete => &COMPLETE_COLOR,
            Completion::AutoHinted(_) => &AUTO_HINT_COLOR,
        }
    }

    pub fn known_characters<'a, 'w>(&'a self, word: &'w Word) -> Option<&'w [Character]> {
        match self {
            Completion::Unstarted => None,
            Completion::Complete => Some(&word.characters),
            Completion::AutoHinted(hints) | Completion::Hinted(hints) => Some(
                &word
                    .characters
                    .split_at(*hints.min(&word.characters.len()))
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
    if !chosen.is_changed() {
        return;
    }
    let grid = current_level.level().grid;
    let chars: CharsArray = chosen.0.iter().map(|t| grid[*t]).collect();

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
        &chosen.0,
        word,
        is_first_time,
        &asset_server,
        &size,
        &current_level,
    );

    found_words.calculate_auto_hints(&current_level);

    if is_first_time {
        *chosen = ChosenState::default();
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

    fn calculate_auto_hints(&mut self, level: &CurrentLevel) {
        let level = level.level();

        let mut word_index = 0;

        while let Some(completion) = self.word_completions.get(word_index) {
            if completion.is_hinted() || completion.is_complete() {
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
                            self.word_completions[word_index] = Completion::AutoHinted(hints.get());
                        }
                        Err(_) => {}
                    }
                }
            }

            word_index += 1;
        }
    }
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

    use crate::prelude::{Completion, CurrentLevel, DesignedLevel, FoundWordsState};

    #[test]
    pub fn test_auto_hints() {
        let level = DesignedLevel::from_tsv_line(
            "DNGLHUAOSTRPAIYC	Europe Countries 2	Austria 	Croatia 	Cyprus  	Hungary 	Poland  	Portugal",
        );
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

        assert_eq!(croatia_completion, Completion::AutoHinted(2));
    }
}
