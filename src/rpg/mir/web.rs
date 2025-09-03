use axum::{http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RollRequest {
    dice_expression: Option<String>,
    character: crate::db::models::Character,
}

pub async fn roll_for(Json(req): Json<RollRequest>) -> Result<Json<(String, f64)>, StatusCode> {
    println!("{:#?}", &req.character);

    match super::roll_with_char_sheet(None, req.dice_expression, &req.character).await {
        Ok(res) => Ok(Json(res)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
