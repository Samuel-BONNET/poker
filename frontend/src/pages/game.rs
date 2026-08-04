use leptos::prelude::*;
use leptos_router::components::*;
use crate::api::ws;
use crate::utils::card::card_image;

#[component]
pub fn GamePage() -> impl IntoView {
    let (raise_value, set_raise_value) = signal(String::new());
    let (show_hand, set_show_hand) = signal(false);
    let send = move |action_type: String| {
        let value = if action_type == "raise" {
            raise_value.get().parse::<i32>().ok()
        } else {
            None
        };
        ws::action(&action_type, value);
        set_raise_value.set(String::new());
        set_show_hand.set(false);
    };

    let client = use_context::<ws::WsClient>().expect("WsClient not provided");

    view! {
        <h1 class="text-center bold">"Texas Hold'Soul"</h1>
        <div class="flex justify-center gap-5">
            <A href="/">
                <button>"Return to hub"</button>
            </A>
            {move || client.room.get().map(|r| view! { <p class="text-center">{format!("Room: {}", r)}</p> })}
        </div>

        {move || client.error.get().map(|e| view! { <p class="text-red-500">{e}</p> })}
        {move || client.notice.get().map(|n| view! { <p class="text-yellow-500">{n}</p> })}

        {move || match client.game.get() {

            Some(g) => {
                let current = g.current_player;
                view! {
                    <div>
                        <div class="flex flex-col items-center">
                            <p>{format!("Pot: {} | Max bet: {} | Moment: {}", g.pot, g.max_bet, g.moment)}</p>
                        </div>
                        {client.my_seat.get().map(|s| view! { <p class="text-center">{format!("Your seat: {}", s)}</p> })}

                        <div class="flex flex-col items-center">
                            <h3>"Commond Cards :"</h3>
                            <div class="align-items gap-1 flex justify-center">
                                {g.common_card.iter().map(|c| {
                                    view! { <img src=card_image(&c.color, &c.value) class="w-48" /> }
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
                                let is_current = i == current && !p.folded && p.active;
                                view! {
                                    <div class=move || {
                                        if is_current {"font-bold border-2 border-blue-500 p-4 flex flex-col items-center"}
                                        else {"border-2 border-transparent p-4 flex flex-col items-center"} }>
                                        <p class="col-span-3">{p.name.clone()} {label} {status}</p>
                                        <p class="col-start-2">{format!("Bankroll: {} | Bet: {}", p.bankroll, p.bet)}</p>
                                        {
                                            if show_hand.get(){
                                                view! {
                                                    <div class="flex justify-center gap-2">
                                                        { client.my_hand.get().iter().map(|c| {
                                                            view! { <img src=card_image(&c.color, &c.value) class="w-48" /> }
                                                        }).collect_view()}
                                                    </div>
                                                }
                                            }
                                            else{
                                                view! {
                                                    <div class="flex justify-center">
                                                        { (0..2).map(|_| {
                                                            view! { <img src="/cards/back_card.png".to_string() class="w-48" /> }
                                                        }).collect_view()}
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
                    </div>
                }
            }.into_any(),
            None => view! { <div class="text-center">"Waiting..."</div> }.into_any()
        }}
    }
}