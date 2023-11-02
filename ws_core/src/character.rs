use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, EnumIs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, EnumCount, EnumIter, EnumIs)]
pub enum Character {
    Blank,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}

impl Character {
    pub fn as_char(&self)-> char{
        match self{
            Character::Blank =>  ' ',
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



impl TryFrom<char> for Character {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
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
            _ => Err(()),
        }
    }
}
