## Running

```bash
docker-compose up
```

This starts four containers:

- **Redis** — shared redis rate limiting
- **Instance 1** and **Instance 2** — Rust servers
- **Nginx** — load balancer exposed on `localhost:8080`

## Endpoints

All three endpoints return the same response: the `instance_id` of the server that handled the request. Each one implements a different rate limiting algorithm per client IP.

| Endpoint | Algorithm | Config |
|---|---|---|
| `GET /api/v1/fixed-window/instance` | Fixed Window | 10 requests per 60s |
| `GET /api/v1/sliding-window/instance` | Sliding Window Log | 10 requests per 60s |
| `GET /api/v1/token-bucket/instance` | Token Bucket | 10 tokens, refills 1 token every 6s |

## Testing

```bash
curl http://localhost:8080/api/v1/fixed-window/instance
curl http://localhost:8080/api/v1/sliding-window/instance
curl http://localhost:8080/api/v1/token-bucket/instance

```

When the rate limit is exceeded, the server responds with `429 Too Many Requests`.

## Observation

The rate limiting logic includes `print` statements so you can follow what's happening in real time. Check the instance logs to see the current request count, remaining tokens etc.

```bash
docker-compose logs -f rate-limiting-instance-1 rate-limiting-instance-2
```