use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api::game::*;


#[component]
pub fn App() -> impl IntoView {
    let (game, set_game) = signal(None::<GameState>);
    let (raise_value, set_raise_value) = signal(String::new());

    let start = move |_| {
        spawn_local(async move{
            if let Ok(g) = start_game().await{
                set_game.set(Some(g))
            }
        });
    };

    let send = move |action_type: String| {
        let val = raise_value.get();
        let value = if action_type == "raise"{
            val.parse::<i32>().ok()
        } else{
            None
        };
        spawn_local(async move{
            if let Ok(g) = send_action(&action_type, value).await{
                set_game.set(Some(g));
                set_raise_value.set(String::new());
            }
        });
    };

    view! {
        <h1>"Texas Hold'Soul"</h1>
        <button on:click=start>"New game"</button>

        {move || game.get().map(|g| {
            let current = g.current_player;
            view!{
                <div>
                    <p>{format!("Pot: {} | Max bet: {} | Moment: {}", g.pot, g.max_bet, g.moment)}</p>
                </div>

                <div>
                    <h3>"Commond Cards :"</h3>
                    {g.common_card.iter().map(|c| {
                        view! { <span>{format!("[{} {}]", c.value, c.color)}</span>}
                    }).collect_view()}
                </div>

                <div>
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
                        let hand_str = p.hand.cards.iter().map(|c| format!("{} {}", c.value, c.color))
                            .collect::<Vec<_>>()
                            .join(", ");
                        let is_current = i == current && !p.folded && p.active;
                        view! {
                            <div style={if is_current { "font-weight:bold; border:2px solid green;" } else { "" }}>
                                <p>{p.name.clone()} {label} {status}</p>
                                <p>{format!("Bankroll: {} | Bet: {}", p.bankroll, p.bet)}</p>
                                <p>{format!("Hand: {}", hand_str)}</p>
                            </div>
                        }
                    }).collect_view()}
                </div>

                <div>
                    <h3>{format!("Actions (player {})", current+1)}</h3>
                    <button on:click=move |_| send("fold".to_string())>"Fold"</button>
                    <button on:click=move |_| send("check".to_string())>"Check"</button>
                    <button on:click=move |_| send("call".to_string())>"Call"</button>
                    <input type="number" placeholder="Raise Amount" prop:value=raise_value
                        on:input=move |ev| set_raise_value.set(event_target_value(&ev)) />
                    <button on:click=move |_| send("raise".to_string())>"Raise"</button>

                </div>
            }
        })}

    }

}