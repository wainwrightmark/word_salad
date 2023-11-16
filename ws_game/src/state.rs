use crate::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_utils::{CanInitTrackedResource, TrackableResource};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize)]
pub struct FoundWordsState {
    pub found: HashSet<CharsArray>,
    pub unneeded_tiles: GridSet,
    #[serde(skip)] //TODO use a different data structure for serialization
    pub hints: HashMap<CharsArray, Hint>,
}

#[derive(Debug, Clone, Resource, Default, Serialize, Deserialize)]
pub struct Hint {
    pub solution: Solution,
    pub number: usize,
}

impl Hint {
    pub fn is_solve(&self) -> bool {
        self.solution.len() <= self.number
    }
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWords";
}

impl FoundWordsState {

    pub fn hint_set(&self)-> GridSet{
        let mut set = GridSet::default();
        for (word, hint) in self.hints.iter() {
            if self.found.contains(word) {
                continue;
            }

            for tile in hint.solution.iter().take(hint.number){
                set.set_bit(tile, true);
            }
        }
        set
    }

    pub fn is_next_hinted(&self, tile: &Tile, current_chosen: &Solution) -> bool {
        for (word, hint) in self.hints.iter() {
            if self.found.contains(word) {
                continue;
            }

            if hint.number == current_chosen.len() + 1 {
                if hint
                    .solution
                    .iter()
                    .take(hint.number)
                    .eq(current_chosen.iter().chain(std::iter::once(tile)))
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_completion(&self, word: &CharsArray) -> Completion {
        if self.found.contains(word) {
            return Completion::Complete;
        }

        if let Some(hints) = self.hints.get(word) {
            return Completion::Hinted(hints.number);
        }
        return Completion::Incomplete;
    }

    pub fn hint_count(&self) -> usize {
        self.hints.values().map(|x| x.number).sum()
    }

    pub fn try_hint_word(&mut self, current_level: &CurrentLevel, word_index: usize)-> bool{
        let level = current_level.level();

        let Some(word) = level.words.get(word_index) else {return false;};

        if self.found.contains(&word.characters){
            return false;
        }

        match self.hints.entry(word.characters.clone()){
            bevy::utils::hashbrown::hash_map::Entry::Occupied(mut o) => {
                if o.get().number >= word.characters.len(){
                    return false; //already fully hinted
                }
                o.get_mut().number += 1;
                return true;
            }
            bevy::utils::hashbrown::hash_map::Entry::Vacant(v) => {
                match word.find_solution(&level.grid) {
                    Some(solution) => {
                        v.insert(Hint {
                            solution,
                            number: 1,
                        });
                    }
                    None => {
                        warn!("No Solution found for {w} whilst hinting", w = word.text);
                        return false;
                    }
                }

                return true;
            }
        }
    }

    pub fn try_hint(&mut self, current_level: &CurrentLevel) -> bool {
        let level = current_level.level();

        let mut min_hints = usize::MAX;

        'check: for word in level
            .words
            .iter()
            .filter(|w| !self.found.contains(&w.characters))
        {
            if let Some(h) = self.hints.get(&word.characters) {
                if !h.is_solve() {
                    min_hints = min_hints.min(h.number);
                }
            } else {
                min_hints = 0;
                break 'check;
            }
        }

        for word in level
            .words
            .iter()
            .filter(|x| !self.found.contains(&x.characters))
        {
            match self.hints.entry(word.characters.clone()) {
                bevy::utils::hashbrown::hash_map::Entry::Occupied(mut o) => {
                    if o.get().number == min_hints {
                        o.get_mut().number = min_hints + 1;
                        return true;
                    }
                }
                bevy::utils::hashbrown::hash_map::Entry::Vacant(v) => {
                    match word.find_solution(&level.grid) {
                        Some(solution) => {
                            v.insert(Hint {
                                solution,
                                number: 1,
                            });
                        }
                        None => {
                            warn!("No Solution found for {w} whilst hinting", w = word.text)
                        }
                    }

                    return true;
                }
            }
        }

        return false;
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Completion {
    Incomplete,
    Hinted(usize),
    Complete,
}

impl Completion {
    pub fn color(&self) -> &'static Color {
        const INCOMPLETE_COLOR: Color = Color::ALICE_BLUE;
        const HINT_COLOR: Color = Color::rgb(0.5, 0.5, 0.99);
        const COMPLETE_COLOR: Color = Color::GREEN;

        match self {
            Completion::Incomplete => &INCOMPLETE_COLOR,
            Completion::Hinted(_) => &HINT_COLOR,
            Completion::Complete => &COMPLETE_COLOR,
        }
    }
}

fn track_found_words(
    mut commands: Commands,
    mut chosen: ResMut<ChosenState>,
    level: Res<CurrentLevel>,
    level_data: Res<LazyLevelData>,
    mut found_words: ResMut<FoundWordsState>,
    asset_server: Res<AssetServer>,
    size: Res<Size>,
) {
    if chosen.is_changed() {
        let grid = level.level().grid;
        let chars: CharsArray = chosen.0.iter().map(|t| grid[*t]).collect();

        if let Some(word) = level_data.words_map.get(&chars) {
            let is_first_time = !found_words.found.contains(&chars);

            if let Some(last_tile) = chosen.0.last() {
                crate::animated_solutions::animate_solution(
                    &mut commands,
                    *last_tile,
                    word,
                    is_first_time,
                    &asset_server,
                    &size,
                    &level
                );
            }

            if is_first_time {
                found_words.found.insert(chars);

                found_words.unneeded_tiles =
                    level_data.calculate_unneeded_tiles(&found_words.found);

                if chosen
                    .0
                    .iter()
                    .any(|x| found_words.unneeded_tiles.get_bit(x))
                {
                    *chosen = ChosenState::default();
                }
            }
        }
    }
}
