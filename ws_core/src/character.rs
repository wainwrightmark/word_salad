use arrayvec::ArrayVec;
use prime_bag::*;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIs, EnumIter};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    EnumCount,
    EnumIter,
    EnumIs,
    FromPrimitive,
)]
#[repr(u8)]

pub enum Character {
    E = 0,
    T = 1,
    A = 2,
    I = 3,
    N = 4,
    O = 5,
    S = 6,
    H = 7,
    R = 8,
    D = 9,
    L = 10,
    U = 11,
    C = 12,
    M = 13,
    F = 14,
    W = 15,
    Y = 16,
    G = 17,
    P = 18,
    B = 19,
    V = 20,
    K = 21,
    Q = 22,
    J = 23,
    X = 24,
    Z = 25,
    Blank,
}

impl PrimeBagElement for Character {
    fn into_prime_index(&self) -> usize {
        *self as usize
    }

    fn from_prime_index(value: usize) -> Self {
        FromPrimitive::from_usize(value).expect("Could not cast usize to character")
    }
}

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Character {
    pub fn to_tile_string(&self) -> String {
        match self {
            Character::Blank => ' ',
            Character::A => 'A',
            Character::B => 'B',
            Character::C => 'C',
            Character::D => 'D',
            Character::E => 'E',
            Character::F => 'F',
            Character::G => 'G',
            Character::H => 'H',
            Character::I => 'I',
            Character::J => 'J',
            Character::K => 'K',
            Character::L => 'L',
            Character::M => 'M',
            Character::N => 'N',
            Character::O => 'O',
            Character::P => 'P',
            Character::Q => 'Q',
            Character::R => 'R',
            Character::S => 'S',
            Character::T => 'T',
            Character::U => 'U',
            Character::V => 'V',
            Character::W => 'W',
            Character::X => 'X',
            Character::Y => 'Y',
            Character::Z => 'Z',
        }
        .to_string()
    }

    pub fn as_char(&self) -> char {
        match self {
            Character::Blank => '_',
            Character::A => 'A',
            Character::B => 'B',
            Character::C => 'C',
            Character::D => 'D',
            Character::E => 'E',
            Character::F => 'F',
            Character::G => 'G',
            Character::H => 'H',
            Character::I => 'I',
            Character::J => 'J',
            Character::K => 'K',
            Character::L => 'L',
            Character::M => 'M',
            Character::N => 'N',
            Character::O => 'O',
            Character::P => 'P',
            Character::Q => 'Q',
            Character::R => 'R',
            Character::S => 'S',
            Character::T => 'T',
            Character::U => 'U',
            Character::V => 'V',
            Character::W => 'W',
            Character::X => 'X',
            Character::Y => 'Y',
            Character::Z => 'Z',
        }
    }
}

pub fn normalize_characters_array(text: &str) -> Result<ArrayVec<Character, 16>, &'static str> {
    let mut characters = ArrayVec::<Character, 16>::default();
    let unicode_graphemes = unicode_segmentation::UnicodeSegmentation::graphemes(text, true);

    for grapheme in unicode_graphemes {
        let mut normalized = unicode_normalization::UnicodeNormalization::nfd(grapheme);

        let Some(c) = normalized.next() else {
            continue;
        };
        let character = Character::try_from(c)?;
        if !character.is_blank() {
            characters
                .try_push(character)
                .map_err(|_| "Word is too long")?;
        }
    }

    Ok(characters)
}

impl TryFrom<char> for Character {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if value.is_ascii_punctuation() | value.is_ascii_whitespace() {
            return Ok(Character::Blank);
        }

        match value {
            '_' | ' ' => Ok(Character::Blank),
            'a' | 'A' => Ok(Character::A),
            'b' | 'B' => Ok(Character::B),
            'c' | 'C' => Ok(Character::C),
            'd' | 'D' => Ok(Character::D),
            'e' | 'E' => Ok(Character::E),
            'f' | 'F' => Ok(Character::F),
            'g' | 'G' => Ok(Character::G),
            'h' | 'H' => Ok(Character::H),
            'i' | 'I' => Ok(Character::I),
            'j' | 'J' => Ok(Character::J),
            'k' | 'K' => Ok(Character::K),
            'l' | 'L' => Ok(Character::L),
            'm' | 'M' => Ok(Character::M),
            'n' | 'N' => Ok(Character::N),
            'o' | 'O' => Ok(Character::O),
            'p' | 'P' => Ok(Character::P),
            'q' | 'Q' => Ok(Character::Q),
            'r' | 'R' => Ok(Character::R),
            's' | 'S' => Ok(Character::S),
            't' | 'T' => Ok(Character::T),
            'u' | 'U' => Ok(Character::U),
            'v' | 'V' => Ok(Character::V),
            'w' | 'W' => Ok(Character::W),
            'x' | 'X' => Ok(Character::X),
            'y' | 'Y' => Ok(Character::Y),
            'z' | 'Z' => Ok(Character::Z),
            _ => Err("Invalid character"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CharacterMap<T>([T; 27]);

impl<T> CharacterMap<T> {
    pub fn get(&self, c: Character) -> &T {
        &self.0[c.into_prime_index()]
    }

    pub fn get_mut(&mut self, c: Character) -> &mut T {
        &mut self.0[c.into_prime_index()]
    }

    pub fn set(&mut self, c: Character, value: T) {
        self.0[c.into_prime_index()] = value;
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (Character, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, t)| (Character::from_prime_index(i), t))
    }
}

#[cfg(test)]
mod tests {
    use crate::normalize_characters_array;
    use test_case::test_case;
    /* spellchecker:disable */
    #[test_case("abcd", "ab cd")]
    #[test_case("dali", "Dalí")]
    #[test_case("walle", "Wall-E")]
    fn test_equal(expected: &str, actual: &str) {
        let expected_len = expected.len();
        let expected = normalize_characters_array(expected).unwrap();
        let actual = normalize_characters_array(actual).unwrap();

        assert_eq!(actual, expected);
        assert_eq!(actual.len(), expected_len);
    }
}
