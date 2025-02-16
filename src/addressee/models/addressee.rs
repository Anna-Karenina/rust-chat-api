use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Addressee {
    pub id: usize,
    pub uuid: Option<String>,
    pub name: String,
    pub user_name: String,
    pub user_image_url: String,
}

impl Addressee {
    pub fn new(name: String, user_name: String, user_image_url: String) -> Self {
        let mut rng = rand::thread_rng();
        let id = rng.gen();
        let uuid = Some(Uuid::new_v4().to_string());
        Addressee {
            id,
            uuid,
            name,
            user_name,
            user_image_url,
        }
    }
}

#[derive(Deserialize)]

pub struct CreateAddresseeDTO {
    pub name: String,
    pub user_name: String,
    pub user_image_url: String,
}
