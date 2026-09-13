use leptos::prelude::*;
use shared::card::Card;
use crate::components::card::{CardBack, CardView};

#[component]
pub fn SeatView(
    name: String,
    bankroll: i32,
    bet: i32,
    folded: bool,
    all_in: bool,
    small_blind: bool,
    big_blind: bool,
    is_dealer: bool,
    is_turn: bool,
    is_hero: bool,
    hero_cards: Vec<Card>,
    pos: (f64, f64),
    dir: (f64, f64),
) -> impl IntoView {
    let initials = name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .collect::<String>();
    let initials = if initials.is_empty() { "?".to_string() } else { initials };

    let off_x = (dir.0 * 20.0) as i32;
    let off_y = (dir.1 * 20.0) as i32;
    let bet_style = format!("transform: translate(-50%, -50%) translate({}px, {}px);", off_x, off_y);

    let stack_text = if all_in {
        format!("${} · all-in", bankroll)
    } else {
        format!("${}", bankroll)
    };

    let card_size = "w-28";
    let card_gap = if is_hero { "gap-1.5" } else { "gap-1" };

    let avatar_cls = move || {
        if is_turn {
            "border-accent turn-ring".to_string()
        } else if is_hero {
            "border-white/25".to_string()
        } else {
            "border-white/10".to_string()
        }
    };

    view! {
        <div
            style=format!("left: {:.2}%; top: {:.2}%;", pos.0, pos.1)
            class=move || format!(
                "absolute -translate-x-1/2 -translate-y-1/2 z-10 flex flex-col items-center gap-1 transition-opacity duration-300 {}",
                if folded { "opacity-25" } else { "" }
            )
        >
            {bet.gt(&0).then(|| view! {
                <div style=bet_style class="absolute text-xs text-text/80">{format!("+{}", bet)}</div>
            })}

            <div class="flex gap-1.5 text-[10px] text-muted h-3.5 leading-none">
                {small_blind.then(|| view! { <span>"SB"</span> })}
                {big_blind.then(|| view! { <span>"BB"</span> })}
                {is_dealer.then(|| view! { <span>"BTN"</span> })}
            </div>

            <div class=move || format!(
                "w-9 h-9 rounded-full bg-surface-2 border flex items-center justify-center text-xs font-medium text-text/85 transition-colors duration-200 {}",
                avatar_cls()
            )>{initials}</div>

            <div class="flex flex-col items-center leading-tight">
                <span class=move || format!(
                    "text-xs {} {}",
                    if is_turn { "text-accent" } else if is_hero { "text-text" } else { "text-text/80" },
                    if folded { "line-through" } else { "" }
                )>{name}</span>
                <span class="text-[11px] text-muted">{stack_text}</span>
            </div>

            <div class=format!("flex {}", card_gap)>
                {if is_hero {
                    hero_cards.iter().map(|c| view! { <CardView card=c.clone() class=card_size /> }.into_any()).collect_view()
                } else {
                    (0..2).map(|_| view! { <CardBack class=card_size /> }.into_any()).collect_view()
                }}
            </div>
        </div>
    }
}
