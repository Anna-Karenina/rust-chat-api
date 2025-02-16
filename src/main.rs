pub mod addressee;
pub mod post;

use post::{
    models::chat_room::{ChatRoom, Post},
    routes::{post_box, post_box_create},
};

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount("/", rocket::routes![post_box, post_box_create])
        .manage(Post::default())
        .manage(ChatRoom::default())
        .launch()
        .await;
}
