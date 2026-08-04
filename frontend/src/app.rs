use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use crate::pages::home::HomePage;
use crate::pages::game::GamePage;
use crate::pages::room::RoomPage;
use crate::api::ws;

#[component]
pub fn App() -> impl IntoView {
    let client = ws::client().expect("Cannot connect to server");
    provide_context(client);

    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/game") view=GamePage />
                <Route path=path!("/room") view=RoomPage />
            </Routes>
        </Router>
    }
}