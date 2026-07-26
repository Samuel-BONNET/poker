use leptos::prelude::*;
use leptos_router::components::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <h1>Home</h1>

        <A href="/game">
            <button>"Start a game"</button>
        </A>
    }
}