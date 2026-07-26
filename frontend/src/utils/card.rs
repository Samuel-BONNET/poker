use shared::color::Color;
use shared::value::Value;

pub fn card_image(color: &Color, value: &Value) -> String {
    format!("/cards/{}_{}.png", color.get_name_color(), value.symbol())
}