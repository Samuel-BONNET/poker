use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}


impl Color {
    pub fn get_name_color(&self) -> String{
        match self{
            Color::Clubs => "clubs".to_string(),
            Color::Diamonds => "diamonds".to_string(),
            Color::Hearts => "hearts".to_string(),
            Color::Spades => "spades".to_string(),
        }
    }
}
