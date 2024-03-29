use std::{num::NonZeroUsize, str::FromStr};
use ustr::Ustr;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayWord {
    /// The characters needed to solve the word
    pub characters: CharsArray,
    /// The final display text of the word
    pub text: Ustr,
    /// The text when the word is hidden
    pub hidden_text: Ustr,
    /// The graphemes - used for partially hiding the word
    pub graphemes: Vec<CharGrapheme>,
}

impl std::fmt::Display for DisplayWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharGrapheme {
    pub is_game_char: bool,
    pub grapheme: Ustr,
}

impl WordTrait for DisplayWord {
    fn characters(&self) -> &CharsArray {
        &self.characters
    }
}

impl DisplayWord {
    pub fn hinted_text(&self, hints: NonZeroUsize) -> String {
        let mut result: String = Default::default();
        let mut hints_left = hints.get();

        for grapheme in self.graphemes.iter() {
            if !grapheme.is_game_char || hints_left > 0 {
                result.push_str(grapheme.grapheme.as_str());
                if grapheme.is_game_char {
                    hints_left = hints_left.saturating_sub(1);
                }
            } else {
                result.push_str(" _");
            }
        }

        result
    }
}

impl PartialOrd for DisplayWord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DisplayWord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.characters
            .iter()
            .map(|x| x.as_char())
            .cmp(other.characters.iter().map(|x| x.as_char()))
    }
}

impl FromStr for DisplayWord {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut hidden_text: String = Default::default();
        let mut graphemes: Vec<CharGrapheme> = Default::default();
        let mut stack: usize = 0;
        let mut characters: ArrayVec<Character, 16> = Default::default();

        let unicode_graphemes = unicode_segmentation::UnicodeSegmentation::graphemes(s, true);

        for grapheme in unicode_graphemes {
            let mut normalized = unicode_normalization::UnicodeNormalization::nfd(grapheme);

            let Some(c) = normalized.next() else {
                continue;
            };

            let character = Character::try_from(c)?;

            if character.is_blank() {
                if let Some(char_to_push) = {
                    if ['-', '‐', '–', '—'].contains(&c) {
                        Some('-')
                    } else if c.is_ascii_whitespace() {
                        Some(',') //use a comma instead of a space, like a crossword clue
                    } else {
                        None
                    }
                } {
                    if stack > 0 {
                        hidden_text += stack.to_string().as_str();
                        stack = 0;
                    }
                    hidden_text.push(char_to_push);
                }

                // otherwise ignore the character in the hidden text
            } else {
                characters
                    .try_push(character)
                    .map_err(|_| "Word is too long")?;
                stack += 1;
            }

            graphemes.push(CharGrapheme {
                is_game_char: !character.is_blank(),
                grapheme: Ustr::from(grapheme),
            })
        }
        if stack > 0 {
            hidden_text += stack.to_string().as_str();
        }

        if characters.len() <= 3 {
            return Err("Word has 3 or fewer characters");
        }

        Ok(Self {
            characters,
            text: Ustr::from(s),
            hidden_text: Ustr::from(&hidden_text),
            graphemes,
        })
    }
}
