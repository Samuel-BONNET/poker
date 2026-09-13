use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::api::ws;

fn initials(name: &str) -> String {
    name.split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .map(|c| c.to_uppercase().to_string())
        .collect::<String>()
}

#[component]
pub fn RoomPage() -> impl IntoView {
    let client = use_context::<ws::WsClient>().expect("WsClient not provided");
    let (copied, set_copied) = signal(false);

    let copy = move |_| {
        if let Some(room) = client.room.get() {
            if let Some(nav) = web_sys::window().map(|w| w.navigator().clipboard()) {
                let _ = nav.write_text(&room);
                set_copied.set(true);
            }
        }
    };

    Effect::new(move |_| {
        if copied.get() {
            if let Some(window) = web_sys::window() {
                let set_copied = set_copied.clone();
                let cb = Closure::once(move || set_copied.set(false));
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(),
                    1500,
                );
                cb.forget();
            }
        }
    });

    let leave = move |_| {
        ws::leave();
    };

    view! {
        <div class="h-screen w-screen bg-bg flex items-center justify-center overflow-hidden">
            <div class="w-[24rem] max-w-full mx-4 flex flex-col gap-6 animate-fade-up">
                <div class="flex items-center justify-between">
                    <h1 class="text-lg font-semibold tracking-tight">"Salle d'attente"</h1>
                    <div class="flex items-center gap-2">
                        <span class="text-sm text-muted">"#" {move || client.room.get().unwrap_or_default()}</span>
                        <button class="btn-secondary px-2.5 py-1 text-xs" on:click=copy>
                            {move || if copied.get() { "Copié".to_string() } else { "Copier".to_string() }}
                        </button>
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    <div class="text-xs text-muted uppercase tracking-wide">
                        {move || {
                            let count = client.game.get().map(|g| g.players.iter().filter(|p| p.active).count()).unwrap_or(0);
                            format!("Joueurs · {}", count)
                        }}
                    </div>
                    <div class="flex flex-col gap-2">
                        {move || client.game.get().map(|g| {
                            g.players.iter().enumerate().filter(|(_, p)| p.active).map(|(i, p)| {
                                let is_leader = g.leader_seat == Some(i);
                                view! {
                                    <div class="flex items-center gap-3 bg-surface rounded-lg px-3 py-2.5 border border-line">
                                        <div class="w-8 h-8 rounded-full bg-surface-2 border border-white/10 flex items-center justify-center text-xs font-medium text-text/85">
                                            {initials(&p.name)}
                                        </div>
                                        <div class="flex-1 flex items-center gap-2">
                                            <span class="text-sm text-text">{p.name.clone()}</span>
                                            {is_leader.then(|| view! { <span class="text-[10px] text-muted">"Hôte"</span> })}
                                        </div>
                                    </div>
                                }
                            }).collect_view()
                        })}
                    </div>
                </div>

                {move || if client.is_leader.get() {
                    let players = client.game.get().map(|g| g.players.iter().filter(|p| p.active).count()).unwrap_or(0);
                    view! {
                        <button class="btn-primary py-2.5 w-full" disabled=move || players < 2 on:click=move |_| ws::start()>
                            {move || if players < 2 { format!("En attente de joueurs… ({}/2)", players) } else { "Commencer la partie".to_string() }}
                        </button>
                    }.into_any()
                } else {
                    view! {
                        <div class="text-sm text-muted text-center py-2">"En attente de l'hôte…"</div>
                    }.into_any()
                }}

                <button class="text-sm text-muted hover:text-text transition-colors" on:click=leave>"Quitter la salle"</button>

                <div class="flex flex-col gap-2">
                    {move || client.error.get().map(|e| view! { <div class="animate-fade-up text-sm text-danger bg-danger/10 border border-danger/20 px-3 py-2 rounded-lg">{e}</div> })}
                    {move || client.notice.get().map(|n| view! { <div class="animate-fade-up text-sm text-text bg-surface border border-line px-3 py-2 rounded-lg">{n}</div> })}
                </div>
            </div>
        </div>
    }
}
