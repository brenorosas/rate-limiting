use std::sync::Arc;

use axum::{Extension, Json, debug_handler, http::StatusCode};

use crate::services::instance::{
    InstanceService, dtos::get_instance_response_dto::GetInstanceResponseDto,
};

#[debug_handler]
pub async fn get_instance_handler(
    Extension(instance_service): Extension<Arc<InstanceService>>,
) -> Result<Json<GetInstanceResponseDto>, StatusCode> {
    let response = instance_service.get_instance();

    Ok(Json(response))
}
