#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank {
    HighCard,
    Pair,
    DoublePair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl Rank {
    pub const ALL: [Rank; 9] = [
        Rank::HighCard,
        Rank::Pair,
        Rank::DoublePair,
        Rank::ThreeOfAKind,
        Rank::Straight,
        Rank::Flush,
        Rank::FullHouse,
        Rank::FourOfAKind,
        Rank::StraightFlush,
    ];

    pub fn get_number_rank(&self) -> i32 {
        match self {
            Rank::HighCard => 1,
            Rank::Pair => 2,
            Rank::DoublePair => 3,
            Rank::ThreeOfAKind => 4,
            Rank::Straight => 5,
            Rank::Flush => 6,
            Rank::FullHouse => 7,
            Rank::FourOfAKind => 8,
            Rank::StraightFlush => 9,
        }
    }
}