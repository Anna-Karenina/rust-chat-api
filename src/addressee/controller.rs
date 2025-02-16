use super::models::addressee::{Addressee, CreateAddresseeDTO};
use crate::internal::{
    api_response::ApiResponse,
    cache::{CacheError, CacheService},
};
use rocket::{
    get,
    http::Status,
    post,
    serde::json::{json, Json},
    State,
};

#[post("/addressee", format = "json", data = "<value>")]
pub async fn set_value(
    value: Json<CreateAddresseeDTO>,
    cache_service: &State<CacheService>,
) -> Result<Json<ApiResponse>, CacheError> {
    let addressee = Addressee::new(
        value.name.clone(),
        value.user_name.clone(),
        value.user_image_url.clone(),
    );

    let key = addressee.uuid.clone().unwrap();
    let serialized_value = serde_json::to_string(&addressee)?;
    cache_service.set_value(&key, &serialized_value).await?;
    Ok(Json(ApiResponse {
        data: json!(addressee),
        status: Status::Ok,
    }))
}

#[get("/addressee/<key>")]
pub async fn get_value(
    key: String,
    cache_service: &State<CacheService>,
) -> Result<Json<ApiResponse>, CacheError> {
    let addressee: Addressee = cache_service
        .get_value(&key)
        .await
        .map_err(|e| CacheError::from(e))?;
    Ok(Json(ApiResponse {
        data: json!(addressee),
        status: Status::Ok,
    }))
}
