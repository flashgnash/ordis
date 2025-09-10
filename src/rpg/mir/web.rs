use crate::dice::RollResult;
use axum::{extract::Path, http::StatusCode, Json};

pub async fn roll_for_internal(
    char_id: i32,
    roll_expression: Option<String>,
) -> Result<Json<RollResult>, StatusCode> {
    println!(
        "Roll request received for: {}, roll: {:#?}",
        char_id, roll_expression
    );

    if let Ok(char) = crate::db::characters::get(char_id) {
        match super::roll_with_char_sheet(None, roll_expression, &char).await {
            Ok(res) => Ok(Json(res)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Axum does not allow using Option<> for optional parameters.
// the only way I can achieve this is with two separate handler methods

pub async fn roll_default_for(Path(char_id): Path<i32>) -> Result<Json<RollResult>, StatusCode> {
    roll_for_internal(char_id, None).await
}

pub async fn roll_for(Path(params): Path<(i32, String)>) -> Result<Json<RollResult>, StatusCode> {
    roll_for_internal(params.0, Some(params.1)).await
}
