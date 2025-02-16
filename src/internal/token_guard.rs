use rocket::http::Status;
use rocket::outcome::Outcome;

use rocket::request::{self, FromRequest, Request};

use super::api_response::ErrorResponse;
pub struct TokenGuard {
    pub token: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TokenGuard {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        if let Some(token) = request.cookies().get_private("t") {
            return Outcome::Success(TokenGuard {
                token: token.value().to_string(),
            });
        }

        let err_resp = ErrorResponse {
            message: "Unauthorized: Token not found".to_string(),
            trace: "Trace information here".to_string(),
        };
        request.local_cache(|| err_resp.clone());
        Outcome::Error((Status::Unauthorized, err_resp))
    }
}
