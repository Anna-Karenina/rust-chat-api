use rocket::http::{CookieJar, Status};
use rocket::serde::json::Json;
use rocket::{futures::StreamExt, State};
use rocket::{get, post};
use rocket_ws::{Channel, Message, WebSocket};
use serde_json::from_str;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::addressee::models::addressee::Addressee;

use super::models::chat_room::{ChatRoom, CreatePostBoxDTO, Post};
use super::models::ws_message::{WebSocketMessage, WebSocketMessageType};

static USER_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[post(
    "/post-box/create",
    format = "application/json",
    data = "<post_box_dto>"
)]
pub async fn post_box_create(
    post_box_dto: Json<CreatePostBoxDTO<'_>>,
    state: &State<Post>,
) -> String {
    let chat_id = state
        .add_office(post_box_dto.name, post_box_dto.public)
        .await;

    format!("chat id: {}", chat_id)
}

#[get("/post-box/<post_box_id>/ws")]
pub fn post_box<'r>(
    post_box_id: &str,
    ws: WebSocket,
    state: &'r State<ChatRoom>,
    cookies: &CookieJar<'_>,
) -> Result<Channel<'r>, Status> {
    let addressee_cookies = cookies.get_private("addressee");

    let addressee = if let Some(addressee_cookies) = addressee_cookies {
        from_str::<Addressee>(&addressee_cookies.value()).ok()
    } else {
        return Err(Status::BadRequest);
    };

    let addressee = match addressee {
        Some(addressee) => addressee,
        None => {
            return Err(Status::BadRequest);
        }
    };

    let channel = ws.channel(move |stream| {
        Box::pin(async move {
            let user_id = USER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
            let (ws_sink, mut ws_stream) = stream.split();

            state.add_connection(addressee.id, ws_sink, addressee).await;
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
    });
    Ok(channel)
}

async fn handle_incoming_message(
    message_contnents: Message,
    state: &State<ChatRoom>,
    _connection_id: usize,
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
                    // WebSocketMessage::ChangeUserName => {
                    //     if let Some(ws_user_name) = ws_message.user_name {
                    //         state.change_user_name(ws_user_name, connection_id).await;
                    //         state.broadcast_users_list().await;
                    //     }
                    // }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
