use std::{convert::Infallible, fmt::Display, str::FromStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Reaction {
    Laugh,
    Anger,
    Heart,
    Up,
    Down,
    Sad,
}

pub static REACTIONS: [Reaction; 6] = [
    Reaction::Heart,
    Reaction::Laugh,
    Reaction::Anger,
    Reaction::Sad,
    Reaction::Up,
    Reaction::Down,
];

impl FromStr for Reaction {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "laugh" => Reaction::Laugh,
            "anger" => Reaction::Anger,
            "love" => Reaction::Heart,
            "up" => Reaction::Up,
            "down" => Reaction::Down,
            "sad" => Reaction::Sad,
            _ => unimplemented!(),
        })
    }
}

impl Display for Reaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reaction::Laugh => write!(f, "laugh"),
            Reaction::Anger => write!(f, "anger"),
            Reaction::Heart => write!(f, "love"),
            Reaction::Up => write!(f, "up"),
            Reaction::Down => write!(f, "down"),
            Reaction::Sad => write!(f, "sad"),
        }
    }
}

impl Reaction {
    pub fn get_emoji(&self) -> &str {
        match self {
            Reaction::Heart => "❤️",
            Reaction::Laugh => "😂",
            Reaction::Anger => "😡",
            Reaction::Sad => "😭",
            Reaction::Up => "👍",
            Reaction::Down => "👎",
        }
    }
}
