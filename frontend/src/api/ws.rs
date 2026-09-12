use leptos::prelude::*;
use shared::card::Card;
use shared::message::{ClientMessage, GameSnapshot, ServerMessage};
use std::cell::RefCell;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, Event, MessageEvent, WebSocket};

const WS_URL: &str = "ws://127.0.0.1:3000/ws";

thread_local! {
    static SOCKET: RefCell<Option<WebSocket>> = const { RefCell::new(None) };
    static CLIENT: RefCell<Option<WsClient>> = const { RefCell::new(None) };
    static RESYNC_CLOSURE: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy)]
pub struct WsClient {
    pub room: RwSignal<Option<String>>,
    pub game: RwSignal<Option<GameSnapshot>>,
    pub my_seat: RwSignal<Option<usize>>,
    pub my_hand: RwSignal<Vec<Card>>,
    pub connected: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub notice: RwSignal<Option<String>>,
    pub is_leader: RwSignal<bool>,
    pub last_state: RwSignal<Option<String>>,
    pub rx_count: RwSignal<u64>,
    pub last_rx: RwSignal<Option<String>>,
}

pub fn client() -> Result<WsClient, String> {
    CLIENT.with(|c| {
        if let Some(c) = c.borrow().as_ref() {
            return Ok(*c);
        }
        Ok(new_client()?)
    })
}

fn new_client() -> Result<WsClient, String> {
    web_sys::console::log_1(&"[poker] frontend build: state-driven (callbacks)".into());
    let ws = WebSocket::new(WS_URL).map_err(|_| "WS init error".to_string())?;

    let client = WsClient {
        room: RwSignal::new(None),
        game: RwSignal::new(None),
        my_seat: RwSignal::new(None),
        my_hand: RwSignal::new(Vec::new()),
        connected: RwSignal::new(false),
        error: RwSignal::new(None),
        notice: RwSignal::new(None),
        is_leader: RwSignal::new(false),
        last_state: RwSignal::new(None),
        rx_count: RwSignal::new(0),
        last_rx: RwSignal::new(None),
    };
    CLIENT.with(|c| *c.borrow_mut() = Some(client));

    let on_open = Closure::wrap(Box::new(move |_: Event| {
        client.connected.set(true);
        web_sys::console::log_1(&"[poker] ws open".into());
    }) as Box<dyn FnMut(Event)>);
    ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    on_open.forget();

    let on_close = Closure::wrap(Box::new(move |_: CloseEvent| {
        client.connected.set(false);
        web_sys::console::log_1(&"[poker] ws closed".into());
    }) as Box<dyn FnMut(CloseEvent)>);
    ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget();

    let on_msg = Closure::wrap(Box::new(move |ev: MessageEvent| {
        if let Some(text) = ev.data().as_string() {
            Client::handle_rx(&client, &text);
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(on_msg.as_ref().unchecked_ref()));
    on_msg.forget();

    SOCKET.with(|s| *s.borrow_mut() = Some(ws));

    let cb = Closure::wrap(Box::new(resync) as Box<dyn FnMut()>);
    let handler = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    RESYNC_CLOSURE.with(|c| *c.borrow_mut() = Some(cb));
    if let Some(w) = web_sys::window() {
        let _ = w.set_interval_with_callback_and_timeout_and_arguments_0(&handler, 1500);
    }

    Ok(client)
}

struct Client;

impl Client {
    fn handle_rx(client: &WsClient, text: &str) {
        let rx = client.rx_count.get() + 1;
        client.rx_count.set(rx);
        client.last_rx.set(Some(format!("#{rx}: {}", text.chars().take(400).collect::<String>())));

        match serde_json::from_str::<ServerMessage>(text) {
            Ok(msg) => client.apply(msg),
            Err(_) => {
                web_sys::console::log_1(&format!("[poker] rx #{rx} UNPARSEABLE: {}", text.chars().take(120).collect::<String>()).into());
            }
        }
    }
}

pub fn create_room(name: Option<&str>) {
    send(ClientMessage::CreateRoom { name: name.map(|s| s.to_string()) });
}

pub fn join_room(room: &str, name: Option<&str>) {
    send(ClientMessage::JoinRoom { room: room.to_string(), name: name.map(|s| s.to_string()) });
}

pub fn start() {
    send(ClientMessage::Start)
}

pub fn action(action_type: &str, value: Option<i32>) {
    send(ClientMessage::Action { action_type: action_type.to_string(), value })
}

fn send(msg: ClientMessage) {
    let Ok(text) = serde_json::to_string(&msg) else { return };
    SOCKET.with(|s| {
        if let Some(ws) = s.borrow().as_ref() {
            let _ = ws.send_with_str(&text);
        }
    });
}

fn resync() {
    let started = CLIENT.with(|c| {
        c.borrow().as_ref().map(|cl| cl.game.get().map(|g| g.started).unwrap_or(false)).unwrap_or(false)
    });
    if !started {
        send(ClientMessage::Resync);
    }
}

pub fn leave() {
    send(ClientMessage::Leave);
}

impl WsClient {
    fn apply(&self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome { room, seat } => {
                self.room.set(Some(room));
                self.my_seat.set(Some(seat));
                self.connected.set(true);
                self.error.set(None);
                self.notice.set(None);
                self.is_leader.set(false);
            }
            ServerMessage::RoomCreated { room, seat } => {
                self.room.set(Some(room));
                self.my_seat.set(Some(seat));
                self.connected.set(true);
                self.error.set(None);
                self.notice.set(None);
                self.is_leader.set(true);
            }
            ServerMessage::GameState(snapshot) => {
                self.game.set(Some(snapshot.clone()));
                self.last_state.set(Some(format!("GameState: started={} players={} moment={}", snapshot.started, snapshot.players.len(), snapshot.moment)));
                web_sys::console::log_1(&format!("[poker] GameState started={} players={} my_seat={:?}", snapshot.started, snapshot.players.len(), self.my_seat.get()).into());
                self.is_leader.set(snapshot.leader_seat == self.my_seat.get());
                self.error.set(None);
                self.notice.set(None);
            }
            ServerMessage::YourHand { cards } => self.my_hand.set(cards),
            ServerMessage::PlayerLeft { seat } => {
                self.notice.set(Some(format!("Seat {} left the table", seat)));
            }
            ServerMessage::Leave => {
                self.room.set(None);
                self.game.set(None);
                self.my_seat.set(None);
                self.my_hand.set(Vec::new());
                self.is_leader.set(false);
                self.connected.set(false);
            }
            ServerMessage::Error { message } => self.error.set(Some(message)),
        }
    }
}