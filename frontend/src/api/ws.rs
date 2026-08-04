use futures_channel::mpsc;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message as WsMessage};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::card::Card;
use shared::message::{ClientMessage, GameSnapshot, ServerMessage};
use std::cell::RefCell;

const WS_URL: &str = "ws://127.0.0.1:3000/ws";

thread_local! {
    static SENDER: RefCell<Option<mpsc::UnboundedSender<ClientMessage>>> = const { RefCell::new(None) };
    static CLIENT: RefCell<Option<WsClient>> = const { RefCell::new(None) };
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
    pub is_leader: RwSignal<bool>
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
    let ws = WebSocket::open(WS_URL).map_err(|e| e.to_string())?;
    let (tx, mut rx) = mpsc::unbounded::<ClientMessage>();
    SENDER.with(|s| *s.borrow_mut() = Some(tx));
    let (mut sink, mut stream) = ws.split();

    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sink.send(WsMessage::Text(text)).await.is_err() {
                break;
            }
        }
    });

    let client = WsClient {
        room: RwSignal::new(None),
        game: RwSignal::new(None),
        my_seat: RwSignal::new(None),
        my_hand: RwSignal::new(Vec::new()),
        connected: RwSignal::new(false),
        error: RwSignal::new(None),
        notice: RwSignal::new(None),
        is_leader: RwSignal::new(false),
    };

    CLIENT.with(|c| *c.borrow_mut() = Some(client));

    spawn_local(async move {
        while let Some(msg) = stream.next().await {
            if let Ok(WsMessage::Text(text)) = msg {
                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                    client.apply(server_msg);
                }
            }
        }
        client.connected.set(false);
    });

    Ok(client)
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
    send(ClientMessage::Action { action_type: action_type.to_string(), value})
}

fn send(msg: ClientMessage) {
    SENDER.with(|s| {
       if let Some(tx) = s.borrow().as_ref() {
           let _ = tx.unbounded_send(msg);
       }
    });
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
            ServerMessage::RoomCreated { room, seat }  => {
                self.room.set(Some(room));
                self.my_seat.set(Some(seat));
                self.connected.set(true);
                self.error.set(None);
                self.notice.set(None);
                self.is_leader.set(true);
            }
            ServerMessage::GameState(snapshot) => {
                self.game.set(Some(snapshot));
                self.error.set(None);
            }
            ServerMessage::YourHand { cards } => self.my_hand.set(cards),
            ServerMessage::PlayerLeft { seat } => {
                self.notice.set(Some(format!("Seat {} left the table", seat)));
            }
            ServerMessage::Error { message } => self.error.set(Some(message)),
        }
    }
}