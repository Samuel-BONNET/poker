use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use crate::pages::home::HomePage;
use crate::pages::game::GamePage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/game") view=GamePage />
            </Routes>
        </Router>
    }
}