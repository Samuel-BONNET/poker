use leptos::prelude::*;
use crate::api::ws;

#[component]
pub fn HomePage() -> impl IntoView {
    let client = use_context::<ws::WsClient>().expect("WsClient not provided");
    let (name, set_name) = signal(String::new());
    let (code, set_code) = signal(String::new());

    let create = move |_| {
        let n = name.get();
        ws::create_room(if n.trim().is_empty() { None } else { Some(n.as_str()) });
    };
    let join = move |_| {
        let n = name.get();
        ws::join_room(&code.get(), if n.trim().is_empty() { None } else { Some(n.as_str()) });
    };

    view! {
        <div class="h-screen w-screen bg-bg flex items-center justify-center overflow-hidden">
            <div class="w-[24rem] max-w-full mx-4 flex flex-col gap-7 animate-fade-up">
                <div>
                    <h1 class="text-2xl font-semibold tracking-tight">"Texas Hold'Soul"</h1>
                    <p class="text-sm text-muted mt-1.5">"Créez une partie ou rejoignez une table."</p>
                </div>

                <div class="flex flex-col gap-3">
                    <input class="input" type="text" placeholder="Pseudo (optionnel)" prop:value=name
                        on:input=move |ev| set_name.set(event_target_value(&ev)) />

                    <button class="btn-primary py-2.5" on:click=create>"Créer une partie"</button>

                    <div class="flex gap-2">
                        <input class="input flex-1" type="text" placeholder="Code de salle" prop:value=code
                            on:input=move |ev| set_code.set(event_target_value(&ev)) />
                        <button class="btn-secondary px-5" disabled=move || code.get().trim().is_empty() on:click=join>"Rejoindre"</button>
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    {move || client.error.get().map(|e| view! { <div class="animate-fade-up text-sm text-danger bg-danger/10 border border-danger/20 px-3 py-2 rounded-lg">{e}</div> })}
                </div>
            </div>
        </div>
    }
}
