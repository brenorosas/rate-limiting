use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;

use crate::redis::RedisClient;

#[derive(Clone)]
pub struct FixedWindowArgs {
    pub max_requests: u64,
    pub window_seconds: u64,
}

#[derive(Clone)]
pub enum RateLimitStrategy {
    FixedWindow(FixedWindowArgs),
}

pub struct RateLimitLayer {
    redis: Arc<RedisClient>,
}

impl RateLimitLayer {
    pub fn new(redis: Arc<RedisClient>) -> Self {
        Self { redis }
    }

    pub fn wrap(&self, route: MethodRouter, strategy: RateLimitStrategy) -> MethodRouter {
        let redis = self.redis.clone();
        route.layer(middleware::from_fn(move |req: Request, next: Next| {
            let redis = redis.clone();
            let strategy = strategy.clone();
            async move { Self::rate_limit(redis, strategy, req, next).await }
        }))
    }

    fn get_client_ip(req: &Request) -> Option<String> {
        req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
    }

    async fn fixed_window_rate_limit(
        redis: Arc<RedisClient>,
        args: FixedWindowArgs,
        req: Request,
        next: Next,
    ) -> Response {
        let path = req.uri().path().to_string();
        let ip = match Self::get_client_ip(&req) {
            Some(ip) => ip,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window = now / args.window_seconds;
        let key = format!("rate_limit:{}:{}:{}", path, ip, window);
        let mut conn = redis.get_connection().await;

        let count: u64 = redis::cmd("INCR")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(1);

        if count == 1 {
            let _: () = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(args.window_seconds as i64)
                .query_async(&mut conn)
                .await
                .unwrap_or(());
        }

        if count > args.max_requests {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }

        next.run(req).await
    }

    async fn rate_limit(
        redis: Arc<RedisClient>,
        strategy: RateLimitStrategy,
        req: Request,
        next: Next,
    ) -> Response {
        match strategy {
            RateLimitStrategy::FixedWindow(args) => {
                Self::fixed_window_rate_limit(redis, args, req, next).await
            }
        }
    }
}
