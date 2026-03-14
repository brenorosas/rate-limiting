use std::sync::Arc;

use axum::{Extension, Router, routing::get};

use crate::{
    http::handlers::get_instance::get_instance_handler, services::instance::InstanceService,
};

pub fn build_routes() -> Router {
    let instance_service = InstanceService::new();
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/instance", get(get_instance_handler))
            .layer(Extension(Arc::new(instance_service))),
    )
}
