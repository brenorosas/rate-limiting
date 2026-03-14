use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;

use serde::{Deserialize, Serialize};

use crate::redis::RedisClient;

#[derive(Clone)]
pub struct FixedWindowArgs {
    pub max_requests: u64,
    pub window_seconds: u64,
}

#[derive(Clone)]
pub struct SlidingWindowArgs {
    pub max_requests: u64,
    pub window_seconds: u64,
}

#[derive(Clone)]
pub struct TokenBucketArgs {
    pub bucket_size: u64,
    pub refill_tokens: u64,
    pub refill_seconds: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TokenBucketState {
    tokens: u64,
    last_refill: u64,
}

#[derive(Clone)]
pub enum RateLimitStrategy {
    FixedWindow(FixedWindowArgs),
    SlidingWindow(SlidingWindowArgs),
    TokenBucket(TokenBucketArgs),
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
        let key = format!("rate_limit:fw:{}:{}:{}", path, ip, window);
        let mut conn = redis.get_connection().await;

        let count: u64 = redis::cmd("INCR")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(1);

        println!("fixed window count: {}", count);

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

    async fn sliding_window_rate_limit(
        redis: Arc<RedisClient>,
        args: SlidingWindowArgs,
        req: Request,
        next: Next,
    ) -> Response {
        let path = req.uri().path().to_string();
        let ip = match Self::get_client_ip(&req) {
            Some(ip) => ip,
            None => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let window_ms = (args.window_seconds as f64) * 1000.0;
        let window_start = now_ms - window_ms;
        let key = format!("rate_limit:swl:{}:{}", path, ip);

        let mut conn = redis.get_connection().await;

        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg(0.0)
            .arg(window_start)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let count: u64 = redis::cmd("ZCARD")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        println!("sliding window count: {}", count);

        if count >= args.max_requests {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }

        let member = format!("{}:{}", now_ms, count);

        let _: () = redis::cmd("ZADD")
            .arg(&key)
            .arg(now_ms)
            .arg(&member)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(args.window_seconds as i64)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        next.run(req).await
    }

    async fn token_bucket_rate_limit(
        redis: Arc<RedisClient>,
        args: TokenBucketArgs,
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

        let key = format!("rate_limit:tb:{}:{}", path, ip);
        let mut conn = redis.get_connection().await;

        let stored: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        let bucket = stored
            .and_then(|s| serde_json::from_str::<TokenBucketState>(&s).ok())
            .unwrap_or(TokenBucketState {
                tokens: args.bucket_size,
                last_refill: now,
            });

        let elapsed = now.saturating_sub(bucket.last_refill);
        let refilled = elapsed * args.refill_tokens / args.refill_seconds;
        let new_tokens = (bucket.tokens + refilled).min(args.bucket_size);

        if new_tokens == 0 {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        }

        let updated = TokenBucketState {
            tokens: new_tokens - 1,
            last_refill: now,
        };

        println!("token bucket count: {:?}", updated);

        let ttl = (args.bucket_size * args.refill_seconds / args.refill_tokens) + 1;

        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(serde_json::to_string(&updated).unwrap())
            .arg("EX")
            .arg(ttl)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

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
            RateLimitStrategy::SlidingWindow(args) => {
                Self::sliding_window_rate_limit(redis, args, req, next).await
            }
            RateLimitStrategy::TokenBucket(args) => {
                Self::token_bucket_rate_limit(redis, args, req, next).await
            }
        }
    }
}
