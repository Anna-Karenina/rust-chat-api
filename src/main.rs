use api::models::{ChatMessage, WebSocketMessage, WebSocketMessageType};
use chrono::Utc;

use rocket::futures::{stream::SplitSink, SinkExt, StreamExt};
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket_ws::{stream::DuplexStream, Channel, Message, WebSocket};
use serde_json::{from_str, json};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

static USER_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct ChatRoomConnection {
    user_name: String,
    sink: SplitSink<DuplexStream, Message>,
}

#[derive(Default)]
struct ChatRoom {
    connections: Mutex<HashMap<usize, ChatRoomConnection>>,
}

impl ChatRoom {
    pub async fn add_connection(&self, id: usize, sink: SplitSink<DuplexStream, Message>) {
        let mut connections = self.connections.lock().await;
        let connection = ChatRoomConnection {
            user_name: format!("User #{}", id),
            sink,
        };
        connections.insert(id, connection);
    }

    pub async fn drop_connection(&self, id: usize) {
        self.connections.lock().await.remove(&id);
    }

    pub async fn broadcast_users_list(&self) {
        let mut connections = self.connections.lock().await;
        let user_list = Some(
            connections
                .keys()
                .map(|id| connections[id].user_name.clone())
                .collect::<Vec<String>>(),
        );

        let ws_message = WebSocketMessage {
            message_type: WebSocketMessageType::UserList,
            message: None,
            user_name: None,
            users: user_list,
        };

        for (_, connection) in connections.iter_mut() {
            let _ = connection
                .sink
                .send(Message::Text(json!(ws_message).to_string()))
                .await;
        }
    }

    pub async fn broadcast(&self, chat_message: ChatMessage) {
        let mut connections = self.connections.lock().await;

        let ws_message = WebSocketMessage {
            message_type: WebSocketMessageType::NewMessage,
            message: Some(chat_message),
            users: None,
            user_name: None,
        };
        for (_, connection) in connections.iter_mut() {
            let _ = connection
                .sink
                .send(Message::Text(json!(ws_message).to_string()))
                .await;
        }
    }

    pub async fn send_user_name(&self, id: usize) {
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(&id) {
            let ws_message = WebSocketMessage {
                message_type: WebSocketMessageType::ChangeUserName,
                message: None,
                users: None,
                user_name: Some(conn.user_name.clone()),
            };

            let _ = conn
                .sink
                .send(Message::Text(json!(ws_message).to_string()))
                .await;
        }
    }

    pub async fn change_user_name(&self, new_user_name: String, id: usize) {
        let result = {
            let mut connections = self.connections.lock().await;
            if let Some(connection) = connections.get_mut(&id) {
                let old_user_name = connection.user_name.clone();
                connection.user_name = new_user_name.clone();
                Some(old_user_name)
            } else {
                None
            }
        };

        if let Some(old_user_name) = result {
            let message = ChatMessage {
                message: format!(
                    "User {} changed user name to {}",
                    old_user_name, new_user_name
                ),
                author: "System".to_string(),
                created_at: Utc::now().naive_utc(),
            };
            Self::broadcast(&self, message).await;
        }
    }
}

#[rocket::get("/")]
fn chat<'r>(ws: WebSocket, state: &'r State<ChatRoom>) -> Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move {
            let user_id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
            let (ws_sink, mut ws_stream) = stream.split();

            state.add_connection(user_id, ws_sink).await;
            state.broadcast_users_list().await;
            state.send_user_name(user_id).await;

            while let Some(message) = ws_stream.next().await {
                if let Ok(message_contnents) = message {
                    handle_incoming_message(message_contnents, &state, user_id).await;
                }
            }

            state.drop_connection(user_id).await;
            state.broadcast_users_list().await;
            Ok(())
        })
    })
}

async fn handle_incoming_message(
    message_contnents: Message,
    state: &State<ChatRoom>,
    connection_id: usize,
) {
    match message_contnents {
        Message::Text(json) => {
            if let Ok(ws_message) = from_str::<WebSocketMessage>(&json) {
                match ws_message.message_type {
                    WebSocketMessageType::NewMessage => {
                        if let Some(ws_msg) = ws_message.message {
                            state.broadcast(ws_msg).await;
                        }
                    }
                    WebSocketMessageType::ChangeUserName => {
                        if let Some(ws_user_name) = ws_message.user_name {
                            state.change_user_name(ws_user_name, connection_id).await;
                            state.broadcast_users_list().await;
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![chat])
        .manage(ChatRoom::default())
        .launch()
        .await;
}
