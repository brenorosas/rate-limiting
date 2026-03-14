use std::sync::Arc;

use axum::{Extension, Json, debug_handler, http::StatusCode};

use crate::{
    http::app_state::AppState,
    services::instance::dtos::get_instance_response_dto::GetInstanceResponseDto,
};

#[debug_handler]
pub async fn get_instance_handler(
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<Json<GetInstanceResponseDto>, StatusCode> {
    let response = app_state.instance_service.get_instance();

    Ok(Json(response))
}
