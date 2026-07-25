use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    response: String,
}

pub async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        response: "ok".to_string(),
    })
}