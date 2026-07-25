use serde::Serialize;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Moment {
    Preflop,
    Flop,
    Turn,
    River,
}

impl Moment{

    pub fn next(self) -> Self {
        match self{
            Moment::Preflop => Moment::Flop,
            Moment::Flop => Moment::Turn,
            Moment::Turn => Moment::River,
            Moment::River => Moment::River,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&self) -> Self {
        match self{
            Moment::Preflop => Moment::Preflop,
            Moment::Flop => Moment::Preflop,
            Moment::Turn => Moment::Preflop,
            Moment::River => Moment::Preflop,
        }
    }
}