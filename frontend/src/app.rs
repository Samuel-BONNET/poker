use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
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
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/game") view=GamePage />
                <Route path=path!("/room") view=RoomPage />
            </Routes>
        </Router>
    }.into_any()
}