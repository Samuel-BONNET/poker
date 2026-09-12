use leptos::prelude::*;
use crate::pages::home::HomePage;
use crate::pages::game::GamePage;
use crate::pages::room::RoomPage;
use crate::api::ws;

#[component]
pub fn App() -> impl IntoView {
    let client = match ws::client() {
        Ok(c) => c,
        Err(e) => {
            return view! { <p class="text-red-500">{format!("Connexion impossible : {e}")}</p> }.into_any();
        }
    };
    provide_context(client);
    view! {
        {move || {
            if client.room.get().is_none() {
                view! { <HomePage /> }.into_any()
            } else if client.game.get().map(|g| g.started).unwrap_or(false) {
                view! { <GamePage /> }.into_any()
            } else {
                view! { <RoomPage /> }.into_any()
            }
        }}
    }.into_any()
}