use super::color::Color;
use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub value: Value,
    pub color: Color,
}

impl Card {
    pub fn new(value: Value, color: Color) -> Self{
        Card {
            value,
            color,
        }
    }
}