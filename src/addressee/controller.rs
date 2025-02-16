use crate::internal::{
    api_response::{ApiResponse, ErrorResponse},
    cache::{CacheError, CacheService},
    token_guard::TokenGuard,
};
use rocket::{
    get,
    http::Status,
    post,
    serde::json::{json, Json},
    State,
};

use super::models::addressee::{Addressee, CreateAddresseeDTO};

#[post("/addressee", format = "json", data = "<value>")]
pub async fn create_addressee(
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
pub async fn get_addressee(
    key: &str,
    cache_service: &State<CacheService>,
    _tg: TokenGuard,
) -> Result<Json<ApiResponse>, Json<ErrorResponse>> {
    let addressee: Addressee = cache_service.get_value(&key).await.map_err(|e| {
        let err = ErrorResponse {
            message: "Fail".to_string(),
            trace: CacheError::from(e).to_string(),
        };
        Json(err)
    })?;
    Ok(Json(ApiResponse {
        data: json!(addressee),
        status: Status::Ok,
    }))
}
