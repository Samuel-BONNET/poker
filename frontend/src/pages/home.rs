use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use crate::api::ws;

#[component]
pub fn HomePage() -> impl IntoView {
    let client = use_context::<ws::WsClient>().expect("WsClient not provided");
    let (name, set_name) = signal(String::new());
    let (code, set_code) = signal(String::new());
    let navigate = use_navigate();

    let create = {
        let navigate = navigate.clone();
        move |_| {
            let n = name.get();
            ws::create_room(if n.trim().is_empty() { None } else { Some(n.as_str()) });
            let navigate = navigate.clone();
            Effect::new(move |_| {
                if client.room.get().is_some() {
                    navigate("/room", Default::default());
                }
            });
        }
    };
    let join = {
        let navigate = navigate.clone();
        move |_| {
            let n = name.get();
            ws::join_room(&code.get(), if n.trim().is_empty() { None } else { Some(n.as_str()) });
            let navigate = navigate.clone();
            Effect::new(move |_| {
                if client.room.get().is_some() {
                    navigate("/room", Default::default());
                }
            });
        }
    };

    view! {
        <h1>"Texas Hold'Soul"</h1>

        <div>
            <input type="text" placeholder="Your name" prop:value=name on:input=move |ev| set_name.set(event_target_value(&ev)) />
            <button on:click=create>"Create a game"</button>

            <input type="text" placeholder="Room code" prop:value=code on:input=move |ev| set_code.set(event_target_value(&ev)) />
            <button on:click=join>"Join a game"</button>
        </div>
        {move || client.room.get().map(|r| view! { <p class="text-center">{format!("Room: {}", r)}</p> })}
        {move || client.error.get().map(|e| view! { <p class="text-red-500">{e}</p> })}
    }
}