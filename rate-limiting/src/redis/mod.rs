use redis::Client;
use redis::aio::MultiplexedConnection;

#[derive(Clone)]
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
        let client = Client::open(redis_url).expect("Invalid Redis URL");
        Self { client }
    }

    pub async fn get_connection(&self) -> MultiplexedConnection {
        self.client
            .get_multiplexed_async_connection()
            .await
            .expect("Failed to connect to Redis")
    }
}
