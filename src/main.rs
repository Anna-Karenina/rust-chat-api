pub mod addressee;
pub mod internal;
pub mod post;
pub mod session;

use internal::{api_response::ErrorResponse, cache::CacheService};

use addressee::controller::{create_addressee, get_addressee};
use post::{
    controller::{post_box, post_box_create},
    models::chat_room::{ChatRoom, Post},
};

use rocket::{catch, catchers, serde::json::Json, Request};
use session::controller::{create_session, logout};

#[catch(401)]
pub fn unauthorized(request: &Request) -> Json<ErrorResponse> {
    let err_resp = request.local_cache(|| ErrorResponse {
        message: "Unauthorized: Token not found".to_string(),
        trace: "Default trace information".to_string(),
    });
    Json(err_resp.clone())
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = "redis://127.0.0.1/";
    let cache_service = CacheService::new(redis_url).expect("Failed to create CacheService");

    let _ = rocket::build()
        .manage(cache_service)
        .manage(Post::default())
        .manage(ChatRoom::default())
        .mount("/session", rocket::routes![create_session, logout])
        .mount("/post-box", rocket::routes![post_box, post_box_create])
        .mount("/", rocket::routes![create_addressee, get_addressee])
        .register("/", catchers![unauthorized])
        .launch()
        .await;
    Ok(())
}
