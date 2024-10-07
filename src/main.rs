mod copy;

use axum::{
    routing::get,
    Router,
    extract::Path,
    response::IntoResponse,
    http::StatusCode,
};
use copy::Args;
use tokio::task;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn fetch_repo(Path((org, repo)): Path<(String, String)>) -> impl IntoResponse {
    let url = format!("https://github.com/{}/{}", org, repo);

    // Use tokio's spawn_blocking to run the synchronous code in a separate thread
    let result = task::spawn_blocking(move || {
        let args = Args {
            url,
            timeout: 30,  // Increased timeout for potentially larger repos
        };

        copy::main(args)
    }).await;

    match result {
        Ok(Ok(content)) => (StatusCode::OK, content).into_response(),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)).into_response(),
    }
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/:org/:repo", get(fetch_repo));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}