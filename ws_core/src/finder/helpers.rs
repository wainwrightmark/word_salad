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

pub struct GridSetIterator<const WIDTH: u8, const HEIGHT: u8, const SIZE: usize>
{
    inner: u16,
    last: u8
}

impl<const WIDTH: u8, const HEIGHT: u8, const SIZE: usize> ExactSizeIterator for GridSetIterator<WIDTH, HEIGHT, SIZE> {
    fn len(&self) -> usize {
        self.inner.count_zeros() as usize
    }
}

impl<const WIDTH: u8, const HEIGHT: u8, const SIZE: usize> Iterator for GridSetIterator<WIDTH, HEIGHT, SIZE> {
    type Item = geometrid::tile::Tile<WIDTH, HEIGHT>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner == 0{
            return None;
        }
        let zeros = self.inner.trailing_zeros();
        self.inner = self.inner.wrapping_shr(zeros + 1);
        let ret =  geometrid::tile::Tile::<WIDTH, HEIGHT>::try_from_inner(self.last + zeros as u8);
        self.last += (zeros + 1) as u8;
        ret
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.inner.count_zeros() as usize;
        (size, Some(size))
    }
}

impl<const WIDTH: u8, const HEIGHT: u8, const SIZE: usize> GridSetIterator<WIDTH, HEIGHT, SIZE> {
    pub fn new(grid: &TileSet16<WIDTH, HEIGHT, SIZE>) -> Self { Self { inner: grid.into_inner(), last: 0 } }
}


pub fn iter_true<const WIDTH: u8, const HEIGHT: u8, const SIZE: usize>(grid: &TileSet16<WIDTH, HEIGHT, SIZE>)-> impl Iterator<Item = geometrid::tile::Tile<WIDTH, HEIGHT>> + ExactSizeIterator{
    GridSetIterator::new(grid)
}

#[cfg(test)]
mod tests{
    use crate::*;

    use super::iter_true;

    #[test]
    pub fn test_grid_set_iterator(){
        let set = GridSet::from_fn(|t| t.is_adjacent_to(&Tile::new_const::<2,2>()));

        println!("{set}");

        let iterator = iter_true(&set);

        assert_eq!(8, iterator.len());

        let new_set = GridSet::from_iter(iterator);

        println!("");
        println!("{new_set}");

        assert_eq!(set, new_set)

    }
}