use leptos::prelude::*;
use crate::api::ws;
use crate::components::action_bar::ActionBar;
use crate::components::card::CardView;
use crate::components::seat::SeatView;

fn moment_label(m: &str) -> String {
    match m {
        "Preflop" => "Préflop".to_string(),
        "Flop" => "Flop".to_string(),
        "Turn" => "Turn".to_string(),
        "River" => "River".to_string(),
        other => other.to_string(),
    }
}

fn seat_position(n: usize, my_seat: usize, index: usize) -> (f64, f64, f64, f64) {
    let count = n.max(2) as f64;
    let k = (index + n - (my_seat % n)) % n;
    let theta = std::f64::consts::FRAC_PI_2 + (k as f64 * 2.0 * std::f64::consts::PI / count);
    let ry = if k == 0 { 26.0 } else { 40.0 };
    (
        50.0 + 44.0 * theta.cos(),
        50.0 + ry * theta.sin(),
        -theta.cos(),
        -theta.sin(),
    )
}

#[component]
pub fn GamePage() -> impl IntoView {
    let client = use_context::<ws::WsClient>().expect("WsClient not provided");

    let my_turn = Signal::derive(move || {
        client.my_seat.get().is_some_and(|seat| {
            client.game.get().is_some_and(|g| {
                seat == g.current_player
                    && !g.run_out
                    && g.players.get(seat).is_some_and(|p| p.active && !p.folded && !p.all_in)
            })
        })
    });

    let (pot_anim, set_pot_anim) = signal(false);
    Effect::new(move |_| {
        let _ = client.game.get().map(|g| g.pot);
        set_pot_anim.set(false);
        queue_microtask(move || set_pot_anim.set(true));
    });

    view! {
        <div class="h-screen w-screen overflow-hidden bg-bg flex flex-col relative">
            <header class="flex items-center justify-between px-6 py-4 z-30">
                <button class="btn-secondary px-3 py-1.5 text-xs" on:click=move |_| ws::leave()>"← Retour"</button>
                <div class="flex items-center gap-3 text-sm text-muted">
                    {move || client.room.get().map(|r| view! { <span>{format!("Salle {}", r)}</span> })}
                    {move || client.game.get().map(|g| view! { <span class="text-text/80">{moment_label(&g.moment)}</span> })}
                </div>
                <div class="w-16"></div>
            </header>

            <div class="absolute top-16 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center gap-2 w-full px-4 pointer-events-none">
                {move || client.error.get().map(|e| view! { <div class="animate-fade-up text-sm text-danger bg-danger/10 border border-danger/20 px-3 py-2 rounded-lg">{e}</div> })}
                {move || client.notice.get().map(|n| view! { <div class="animate-fade-up text-sm text-text bg-surface border border-line px-3 py-2 rounded-lg">{n}</div> })}
            </div>

            <main class="relative flex-1">
                <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(88vw,112vh)] aspect-[10/6] rounded-full border border-line bg-surface/40"></div>

                <div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-20 flex flex-col items-center gap-3 pointer-events-none">
                    {move || client.game.get().map(|g| view! {
                        <div class="flex gap-2">
                            {(0..5).map(|j| {
                                if let Some(card) = g.common_card.get(j).cloned() {
                                    view! { <CardView card=card class="w-14" /> }.into_any()
                                } else {
                                    view! { <div class="w-14 aspect-[242/340] rounded-md border border-dashed border-white/10" /> }.into_any()
                                }
                            }).collect_view()}
                        </div>
                        <div class=move || format!(
                            "text-lg font-medium text-text/90 {}",
                            if pot_anim.get() { "pot-pop" } else { "" }
                        )>
                            {format!("Pot · {}", g.pot)}
                        </div>
                    })}
                </div>

                {move || client.game.get().map(|g| view! {
                    <div>
                        {g.run_out.then(|| view! {
                            <div class="absolute left-1/2 top-[15%] -translate-x-1/2 z-30 animate-fade-up text-sm text-muted">"Showdown en cours…"</div>
                        })}
                        {g.last_winner.clone().map(|w| view! {
                            <div class="absolute left-1/2 top-[15%] -translate-x-1/2 z-30 animate-fade-up text-sm text-text/90">{format!("{} remporte le pot", w)}</div>
                        })}
                    </div>
                })}

                {move || client.game.get().map(|g| {
                    let n = g.players.len();
                    let my_seat = client.my_seat.get().unwrap_or(0);
                    let my_hand = client.my_hand.get();
                    let current = g.current_player;
                    g.players.iter().enumerate()
                        .filter(|(_, p)| p.active)
                        .map(|(i, p)| {
                            let (x, y, dx, dy) = seat_position(n, my_seat, i);
                            let is_hero = my_seat == i;
                            let is_turn = !g.run_out && current == i && !p.folded;
                            let hero_cards = if is_hero { my_hand.clone() } else { Vec::new() };
                            view! {
                                <SeatView
                                    name=p.name.clone()
                                    bankroll=p.bankroll
                                    bet=p.bet
                                    folded=p.folded
                                    all_in=p.all_in
                                    small_blind=p.small_blind
                                    big_blind=p.big_blind
                                    is_dealer=i == g.dealer_index
                                    is_turn=is_turn
                                    is_hero=is_hero
                                    hero_cards=hero_cards
                                    pos=(x, y)
                                    dir=(dx, dy)
                                />
                            }
                        }).collect_view()
                })}

                {move || client.game.get().map(|g| {
                    if !g.started {
                        view! { <div class="absolute inset-0 z-40 flex items-center justify-center">
                            <div class="animate-fade-up text-sm text-muted">"En attente du début de la partie…"</div>
                        </div> }.into_any()
                    } else { ().into_any() }
                })}

                {move || client.game.get().is_none().then(|| view! {
                    <div class="absolute inset-0 z-40 flex items-center justify-center">
                        <div class="animate-fade-up text-sm text-muted">"Connexion à la table…"</div>
                    </div>
                })}
            </main>

            <footer class="z-30 flex justify-center px-4 pb-4">
                <ActionBar my_turn=my_turn />
            </footer>
        </div>
    }
}
