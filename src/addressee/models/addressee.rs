use serde::Deserialize;

#[derive(Default, Deserialize)]
pub struct Addressee {
    pub id: usize,
    pub name: String,
    pub user_name: String,
    pub user_image_url: String,
}
