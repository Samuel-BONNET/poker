use crate::color::Color;
use crate::value::Value;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
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