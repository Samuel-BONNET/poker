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
}