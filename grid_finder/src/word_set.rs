use std::ops::{Deref, DerefMut};

use const_sized_bit_set::BitSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord, Hash)]
pub struct WordSet<const W: usize>(pub BitSet<W>);

// impl<const W: usize> core::hash::Hash for WordSet<W> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         let mut data: u64 = 0;
//         for x in self.0.into_inner() {
//             data |= x;
//         }

//         state.write_u64(data);
//     }
// }

impl<const W: usize> DerefMut for WordSet<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const W: usize> Deref for WordSet<W> {
    type Target = BitSet<W>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

