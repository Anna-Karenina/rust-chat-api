use rocket::{
    http::{Cookie, CookieJar, Status},
    serde::json::Json,
    State,
};

use rocket::{delete, put};

use crate::{
    addressee::models::addressee::Addressee,
    internal::{api_response::ApiResponse, cache::CacheService},
};

#[delete("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Status, (Status, Json<ApiResponse>)> {
    cookies.remove_private("a");
    cookies.remove_private("t");
    Ok(Status::Ok)
}

#[put("/<user_uuid>")]
pub async fn create_session<'r>(
    user_uuid: &str,
    state: &'r State<CacheService>,
    cookies: &CookieJar<'_>,
) -> Result<Json<ApiResponse>, (Status, Json<ApiResponse>)> {
    let addressee: Addressee = state.get_value(&user_uuid).await.map_err(|_| {
        (
            Status::NotFound,
            Json(ApiResponse {
                data: serde_json::Value::String("User  not found".to_string()),
                status: Status::NotFound,
            }),
        )
    })?;
    cookies.add_private(Cookie::new("t", String::from("bla")));
    cookies.add_private(Cookie::new("a", serde_json::to_string(&addressee).unwrap()));
    Ok(Json(ApiResponse {
        data: serde_json::Value::String(format!("Hi {}!", addressee.name)),
        status: Status::Ok,
    }))
}
