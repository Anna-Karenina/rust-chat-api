use rocket::{
    http::{ContentType, Status},
    request::Request,
    response::{Responder, Response},
    serde::json::Value,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ErrorResponse {
    pub message: String,
    pub trace: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub data: Value,
    pub status: Status,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        Response::build_from(self.data.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
