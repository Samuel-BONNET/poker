#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}


impl Color {
    pub const ALL: [Color; 4] = [
            Color::Hearts,
            Color::Diamonds,
            Color::Spades,
            Color::Clubs,
    ];
}
