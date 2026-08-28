use leptos::prelude::*;
use shared::card::Card;
use crate::utils::card::card_image;

#[component]
pub fn CardView(card: Card, class: &'static str) -> impl IntoView {
    view! {
        <img src=card_image(&card.color, &card.value) class=format!("animate-deal {} rounded-md shadow-md shadow-black/30", class) alt="carte" />
    }
}

#[component]
pub fn CardBack(class: &'static str) -> impl IntoView {
    view! {
        <img src="/cards/back_card.png" class=format!("animate-deal {} rounded-md shadow-md shadow-black/30", class) alt="dos de carte" />
    }
}
