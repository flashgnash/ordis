use axum::{http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RollRequest {
    dice_expression: Option<String>,
    character: crate::db::models::Character,
}

pub async fn roll_for(Json(req): Json<RollRequest>) -> Result<Json<(String, f64)>, StatusCode> {
    // println!("{:#?}", &req.character);

    if let Some(char_id) = req.character.id {
        println!("ID: {char_id}");

        if let Ok(char) = crate::db::characters::get(char_id) {
            match super::roll_with_char_sheet(None, req.dice_expression, &char).await {
                Ok(res) => Ok(Json(res)),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
