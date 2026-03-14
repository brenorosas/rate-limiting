use std::sync::Arc;

use crate::{
    http::middlewares::rate_limit::RateLimitLayer, redis::RedisClient,
    services::instance::InstanceService,
};

pub struct AppState {
    pub redis: Arc<RedisClient>,
    pub instance_service: Arc<InstanceService>,
    pub rate_limiter: RateLimitLayer,
}

impl AppState {
    pub fn new() -> Self {
        let redis = Arc::new(RedisClient::new());
        let instance_service = Arc::new(InstanceService::new());
        let rate_limiter = RateLimitLayer::new(redis.clone());
        Self {
            redis,
            instance_service,
            rate_limiter,
        }
    }
}
