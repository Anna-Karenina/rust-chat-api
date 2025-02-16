pub mod addressee;
pub mod internal;
pub mod post;

use internal::cache::CacheService;

use addressee::controller::{get_value, set_value};
use post::{
    controller::{post_box, post_box_create},
    models::chat_room::{ChatRoom, Post},
};

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = "redis://127.0.0.1/";
    let cache_service = CacheService::new(redis_url).expect("Failed to create CacheService");

    let _ = rocket::build()
        .manage(cache_service)
        .manage(Post::default())
        .manage(ChatRoom::default())
        .mount(
            "/",
            rocket::routes![post_box, post_box_create, get_value, set_value],
        )
        .launch()
        .await;
    Ok(())
}
