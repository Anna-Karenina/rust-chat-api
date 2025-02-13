use chrono::Utc;
use rocket::futures::{stream::SplitSink, SinkExt};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket_ws::{stream::DuplexStream, Message};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::addressee::models::addressee::Addressee;

use super::ws_message::{ChatMessage, WebSocketMessage, WebSocketMessageType};

#[derive(Default)]
pub struct Post {
    pub offices: Arc<Mutex<HashMap<Uuid, ChatRoom>>>,
}

impl Post {
    pub async fn add_office(&self, name: &str, public: bool) -> Uuid {
        let chat_id = Uuid::new_v4();

        let chat_room = ChatRoom {
            name: name.to_string(),
            public,
            ..Default::default()
        };
        self.offices.lock().await.insert(chat_id, chat_room);
        chat_id
    }
}

pub struct ChatRoomConnection {
    pub addressee: Addressee,
    pub sink: SplitSink<DuplexStream, Message>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct CreatePostBoxDTO<'r> {
    pub public: bool,
    pub name: &'r str,
}

#[derive(Default)]
pub struct ChatRoom {
    pub public: bool,
    pub name: String,
    pub connections: Mutex<HashMap<usize, ChatRoomConnection>>,
}

impl ChatRoom {
    pub async fn add_connection(
        &self,
        id: usize,
        sink: SplitSink<DuplexStream, Message>,
        addressee: Addressee,
    ) {
        let mut connections = self.connections.lock().await;
        let connection = ChatRoomConnection { addressee, sink };
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
                .map(|id| connections[id].addressee.user_name.clone())
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
                user_name: Some(conn.addressee.user_name.clone()),
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
                let old_user_name = connection.addressee.user_name.clone();
                connection.addressee.user_name = new_user_name.clone();
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
