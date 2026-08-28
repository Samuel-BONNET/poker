use leptos::prelude::*;
use crate::api::ws;

#[component]
pub fn ActionBar(my_turn: Signal<bool>) -> impl IntoView {
    let client = use_context::<ws::WsClient>().expect("WsClient not provided");
    let (raise_value, set_raise_value) = signal(1);

    let my_bet = move || client.my_seat.get().and_then(|s| {
        client.game.get().and_then(|g| g.players.get(s).map(|p| p.bet))
    }).unwrap_or(0);
    let my_bankroll = move || client.my_seat.get().and_then(|s| {
        client.game.get().and_then(|g| g.players.get(s).map(|p| p.bankroll))
    }).unwrap_or(0);
    let max_bet = move || client.game.get().map(|g| g.max_bet).unwrap_or(0);

    let call = move || (max_bet() - my_bet()).max(0);
    let raise_hi = Signal::derive(move || (my_bankroll() - call()).max(0));
    let raise_total = move || max_bet() + raise_value.get();
    let can_raise = move || my_turn.get() && raise_hi.get() > 0;

    let was_turn = RwSignal::new(false);
    Effect::new(move |_| {
        let turn = my_turn.get();
        if turn && !was_turn.get() {
            set_raise_value.set(1);
        }
        was_turn.set(turn);
        let hi = raise_hi.get();
        let cur = raise_value.get();
        if hi > 0 && cur < 1 {
            set_raise_value.set(1);
        } else if hi > 0 && cur > hi {
            set_raise_value.set(hi);
        }
    });

    view! {
        <div class="flex items-center gap-2 bg-surface border border-line rounded-2xl px-3 py-2.5">
            <button class="btn-danger px-4 py-2 text-xs" disabled=move || !my_turn.get() on:click=move |_| ws::action("fold", None)>"Fold"</button>

            <button class="btn-primary w-28 px-4 py-2 text-xs" disabled=move || !my_turn.get() on:click=move |_| {
                if call() == 0 { ws::action("check", None) } else { ws::action("call", None) }
            }>
                {move || if call() == 0 { "Check".into_any() } else { format!("Call ${}", call()).into_any() }}
            </button>

            <div class="flex flex-col items-center gap-1 px-2">
                <input
                    type="range"
                    prop:min="1"
                    prop:max=raise_hi
                    prop:value=raise_value
                    disabled=move || !can_raise()
                    class="slider w-36"
                    on:input=move |ev| {
                        if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                            let hi = raise_hi.get();
                            set_raise_value.set(v.clamp(1, if hi > 0 { hi } else { 1 }));
                        }
                    }
                />
                <span class="text-[11px] text-muted">
                    {move || if raise_hi.get() > 0 { format!("Raise +${}", raise_value.get()) } else { "All-in call".to_string() }}
                </span>
            </div>

            <button class="btn-primary w-28 px-4 py-2 text-xs" disabled=move || !can_raise() on:click=move |_| ws::action("raise", Some(raise_value.get()))>
                {move || format!("Raise ${}", raise_total())}
            </button>

            <button class="btn-secondary px-4 py-2 text-xs" disabled=move || !my_turn.get() on:click=move |_| ws::action("all-in", None)>"All-in"</button>
        </div>
    }
}
