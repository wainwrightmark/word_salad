use crate::prelude::*;
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
    pub words: Vec<Completion>,
    pub hints_used: usize,
}

impl Default for FoundWordsState{
    fn default() -> Self {
        Self::new_from_level(&CurrentLevel::default())
    }
}

impl TrackableResource for FoundWordsState {
    const KEY: &'static str = "FoundWords";
}

impl FoundWordsState {

    pub fn new_from_level(current_level: &CurrentLevel)-> Self{
        let level = current_level.level();
        Self { unneeded_tiles: GridSet::EMPTY, words: vec![Completion::Incomplete; level.words.len()], hints_used: 0 }
    }

    /// Grid with unneeded characters blanked
    fn adjusted_grid(&self, level: &CurrentLevel)-> Grid{
        let mut grid = level.level().grid;

        for tile in self.unneeded_tiles.iter_true_tiles(){
            grid[tile] = Character::Blank;
        }

        grid
    }

    pub fn hint_set(&self, level: &CurrentLevel, chosen_state: &ChosenState) -> GridSet {
        let mut set = GridSet::default();
        let adjusted_grid = self.adjusted_grid(level);

        if chosen_state.0.is_empty(){
            //hint all known first letters
            for (word, completion) in level.level().words.iter().zip(self.words.iter()){
                let Completion::Hinted(..) = completion else {continue;};

                if let Some(solution) = word.find_solution(&adjusted_grid){
                    if let Some(first) = solution.first(){
                        set.set_bit(first, true)
                    }
                }
            }
        }
        else{
            // hint all solutions starting with this
            for (word, completion) in level.level().words.iter().zip(self.words.iter()){
                let Completion::Hinted(hints) = completion else {continue;};

                if hints <= &chosen_state.0.len(){continue;};

                if let Some(solution) = word.find_solution(&adjusted_grid){
                    if solution.starts_with(chosen_state.0.as_slice()){
                        if let Some(tile)  = solution.get(chosen_state.0.len()){
                            set.set_bit(tile, true)
                        }
                    }
                }
            }
        }
        set
    }

    pub fn is_level_complete(&self) -> bool {
        self.words.iter().all(|x| x.is_complete())
    }

    pub fn get_completion(&self, word_index: usize) -> Completion {
        self.words
            .get(word_index)
            .unwrap_or(&Completion::Complete)
            .clone()
    }

    pub fn try_hint_word(&mut self, current_level: &CurrentLevel, word_index: usize) -> bool {
        let level = current_level.level();

        let Some(completion) = self.words.get_mut(word_index) else {
            return false;
        };
        let Some(word) = level.words.get(word_index) else {
            return false;
        };

        match completion {
            Completion::Incomplete => {
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
        }
        return true;
    }

    pub fn try_hint(&mut self, current_level: &CurrentLevel) -> bool {
        let level = current_level.level();

        let mut min_hints = usize::MAX;
        let mut min_hint_index: Option<usize> = None;

        'check: for (index, (word, completion)) in
            level.words.iter().zip(self.words.iter()).enumerate()
        {
            match completion {
                Completion::Incomplete => {
                    min_hints = 0;
                    min_hint_index = Some(index);
                    break 'check;
                }
                Completion::Hinted(hints) => {
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
        self.words[index] = Completion::Hinted(min_hints + 1);

        true
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize, EnumIs)]
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

    let Some(completion) = found_words.words.get(word_index) else {
        return;
    };

    let is_first_time = !completion.is_complete();
    if is_first_time {
        found_words.words[word_index] = Completion::Complete;
        *chosen = ChosenState::default();
        found_words.unneeded_tiles =
            level.calculate_unneeded_tiles(found_words.unneeded_tiles, |index| {
                found_words
                    .words
                    .get(index)
                    .map(|x| x.is_complete())
                    .unwrap_or(true)
            });
    }

    let Some(last_tile) = chosen.0.last() else {
        return;
    };
    crate::animated_solutions::animate_solution(
        &mut commands,
        *last_tile,
        word,
        is_first_time,
        &asset_server,
        &size,
        &current_level,
    );
}
