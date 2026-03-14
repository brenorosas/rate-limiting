use std::sync::Arc;

use axum::{Extension, Router, routing::get};

use crate::http::{
    app_state::AppState,
    handlers::get_instance::get_instance_handler,
    middlewares::rate_limit::{FixedWindowArgs, RateLimitStrategy, SlidingWindowArgs},
};

pub fn build_routes(app_state: AppState) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route(
                    "/fixed-window/instance",
                    app_state.rate_limiter.wrap(
                        get(get_instance_handler),
                        RateLimitStrategy::FixedWindow(FixedWindowArgs {
                            max_requests: 10,
                            window_seconds: 60,
                        }),
                    ),
                )
                .route(
                    "/sliding-window/instance",
                    app_state.rate_limiter.wrap(
                        get(get_instance_handler),
                        RateLimitStrategy::SlidingWindow(SlidingWindowArgs {
                            max_requests: 10,
                            window_seconds: 60,
                        }),
                    ),
                ),
        )
        .layer(Extension(Arc::new(app_state)))
}
