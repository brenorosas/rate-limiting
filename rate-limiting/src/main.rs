use rate_limiting::http::{app_state::AppState, routes::build_routes};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    println!("Starting app state ...");
    let app_state = AppState::new();

    println!("Starting server: binding to 0.0.0.0:8080 ...");
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("valid address");

    println!("Serving HTTP on 0.0.0.0:8080");
    if let Err(e) = axum::serve(listener, build_routes(app_state)).await {
        eprintln!("Server error: {:?}", e);
    }
}
