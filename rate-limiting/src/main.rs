use rate_limiting::http::routes::build_routes;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Simple logging using println! (no tracing/log crate configured)
    println!("Starting server: binding to 0.0.0.0:8080 ...");
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("valid address");

    println!("Serving HTTP on 0.0.0.0:8080");
    if let Err(e) = axum::serve(listener, build_routes()).await {
        eprintln!("Server error: {:?}", e);
    }
}
