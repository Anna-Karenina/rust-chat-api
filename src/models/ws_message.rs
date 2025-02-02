use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]

pub enum WebSocketMessageType {
    NewMessage,
    UserList,
    ChangeUserName,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    pub message_type: WebSocketMessageType,
    pub message: Option<ChatMessage>,
    pub users: Option<Vec<String>>,
    pub user_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ChatMessage {
    pub message: String,
    pub author: String,
    pub created_at: NaiveDateTime,
}
