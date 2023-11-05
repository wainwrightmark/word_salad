use std::collections::HashMap;

use itertools::Itertools;
use prime_bag::PrimeBag128;
use crate::prelude::*;

pub type LetterCounts = PrimeBag128<Character>;
pub type WordMultiMap = HashMap<LetterCounts, Vec<FinderWord>>;

pub fn make_words_from_file(text: &'static str) -> WordMultiMap {
    text.lines()
        .flat_map(|x| x.split(','))
        .flat_map(|x| FinderWord::try_new(x))
        .into_group_map_by(|x| x.counts)
}


#[derive(Debug, Clone, PartialEq)]
pub struct FinderWord {
    pub text: &'static str,
    pub array: CharsArray,
    pub counts: PrimeBag128<Character>,
}

impl std::fmt::Display for FinderWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl FinderWord {
    fn try_new(text: &'static str) -> Option<Self> {
        //println!("'{text}'");
        let array = Word::from_str(text).ok().map(|x| x.characters)?;

        let counts: PrimeBag128<Character> = PrimeBag128::try_from_iter(array.iter().cloned())?;
        Some(Self {
            array,
            counts,
            text,
        })
    }
}