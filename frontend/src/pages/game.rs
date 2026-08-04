use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use crate::api::game::*;
use crate::utils::card::card_image;
use shared::card::Card;
use shared::color::Color;
use shared::value::Value;


#[component]
pub fn GamePage() -> impl IntoView {
    let (game, set_game) = signal(None::<GameState>);
    let (raise_value, set_raise_value) = signal(String::new());
    let (show_hand, set_show_hand) = signal(false);

    let start = move |_| {
        spawn_local(async move {
            if let Ok(g) = start_game().await {
                set_game.set(Some(g))
            }
        });
    };

    let send = move |action_type: String| {
        let val = raise_value.get();
        let value = if action_type == "raise" {
            val.parse::<i32>().ok()
        } else {
            None
        };
        spawn_local(async move {
            if let Ok(g) = send_action(&action_type, value).await {
                set_game.set(Some(g));
                set_raise_value.set(String::new());
                set_show_hand.set(false)
            }
        });
    };

    view! {
        <h1 class="text-center bold">"Texas Hold'Soul"</h1>
        <div class="flex justify-center gap-5">
            <A href="/">
                <button>"Return to hub"</button>
            </A>
            <button on:click=start>"New game"</button>
        </div>

        {move || game.get().map(|g| {
            let current = g.current_player;
            view!{
                <div class="flex flex-col items-center">
                    <p>{format!("Pot: {} | Max bet: {} | Moment: {}", g.pot, g.max_bet, g.moment)}</p>
                </div>

                <div class="flex flex-col items-center">
                    <h3>"Commond Cards :"</h3>
                    <div class="align-items gap-1">
                        {g.common_card.iter().map(|c| {
                            view! {
                                <img src=card_image(&c.color, &c.value) class="w-48" />
                            }
                        }).collect_view()}
                    </div>
                </div>

                <div class="flex flex-col items-center">
                    <h3>"Players"</h3>
                    {g.players.iter().enumerate().map(|(i, p)| {
                        let label = if p.small_blind { "(SB)" }
                            else if p.big_blind { "(BB)" }
                            else if i == g.dealer_index { "(BTN)" }
                            else { "" };
                        let status = if p.folded { " - FOLD " }
                            else if p.all_in { " -ALL-IN" }
                            else if !p.active { "-ELIMINATED" }
                            else { "" };
                        let hand = p.hand.cards.clone();
                        let is_current = i == current && !p.folded && p.active;
                        view! {
                            <div class=move || { if is_current {"font-bold border-2 border-blue-500 p-4 flex flex-col items-center"} else {"border-2 border-transparent p-4 flex flex-col items-center"}}>
                                <p class="col-span-3">{p.name.clone()} {label} {status}</p>
                                <p class="col-start-2">{format!("Bankroll: {} | Bet: {}", p.bankroll, p.bet)}</p>
                                {
                                    if is_current && show_hand.get(){
                                        view! {
                                            <div class="flex justify-center gap-2">
                                                { hand.iter().map(|c| {
                                                    view! {
                                                        <img src=card_image(&c.color, &c.value) class="w-48" />
                                                    }
                                                    }).collect_view()
                                                }
                                            </div>
                                        }
                                    }
                                    else{
                                        view! {
                                            <div class="flex justify-center">
                                                { (0..2).map(|c| {
                                                    view! {
                                                        <img src="/cards/back_card.png".to_string() class="w-48" />
                                                    }
                                                    }).collect_view()
                                                }
                                            </div>
                                        }
                                    }
                                }
                            </div>
                        }
                    }).collect_view()}
                </div>

                <div class="flex flex-col gap-2 m-4 items-center">
                    <h3>{format!("Actions (player {})", current+1)}</h3>
                    <div class="flex justify-center gap-2">
                        <button on:click=move |_| send("fold".to_string())>"Fold"</button>
                        <button on:click=move |_| send("check".to_string())>"Check"</button>
                        <button on:click=move |_| send("call".to_string())>"Call"</button>
                        <div class="flex flex-col items-center">
                            <input type="number" placeholder="Raise Amount" prop:value=raise_value
                            on:input=move |ev| set_raise_value.set(event_target_value(&ev)) />
                            <button on:click=move |_| send("raise".to_string())>"Raise"</button>
                        </div>
                        <button on:click=move |_| send("all-in".to_string())>"All-in"</button>
                    </div>
                </div>
                <div class="flex flex-col items-center">
                    <button on:click=move |_| set_show_hand.update(|v| *v = !* v)>
                        {move || if show_hand.get() { "Hide" } else { "Show" }}
                    </button>
                </div>
                }
            })
        }
    }
}