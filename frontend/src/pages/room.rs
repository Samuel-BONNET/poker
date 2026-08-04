use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use crate::api::ws;

#[component]
pub fn RoomPage() -> impl IntoView {

    let client = use_context::<ws::WsClient>().expect("WsClient not provided");
    let navigate = use_navigate();

    Effect::new(move |_| {
        if client.game.get().map(|g| g.started).unwrap_or(false) {
            navigate.clone()("/game", Default::default());
        }
    });

    view! {
        <h1 class="text-center">"Texas Hold'Soul"</h1>

        <A href="/">
            <button>"Return to hub"</button>
        </A>

        <h3>"Room Code:" {move || client.room.get().unwrap_or_default()}</h3>

        <h3>"Players"</h3>
        <ul>
            {move || client.game.get().map(|g| {
                g.players.iter().filter(|p| p.active).map(|p| {
                    view! { <li>{p.name.clone()} </li> }
                }).collect_view()
            }).unwrap_or_default()}
        </ul>

        {move || if client.is_leader.get() {
            view! { <button on:click=move |_| ws::start()>"New game"</button> }.into_any()
        } else {
            view! { <p>"Waiting for the host to start the game..."</p> }.into_any()
        }}

        {move || client.error.get().map(|e| view! { <p class="text-red-500">{e}</p> })}
        {move || client.notice.get().map(|n| view! { <p class="text-yellow-500">{n}</p> })}
    }
}