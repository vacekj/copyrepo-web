mod copy;

use axum::{
    extract::Path,
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use copy::{Args};
use std::path::PathBuf;
use tokio::task;

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn fetch_repo(Path((org, repo)): Path<(String, String)>) -> impl IntoResponse {
    let url = format!("https://github.com/{}/{}/tree/main", org, repo);

    // Use tokio's spawn_blocking to run the synchronous code in a separate thread
    let result = task::spawn_blocking(move || {
        let args = Args {
            url,
            timeout: 30,  // Increased timeout for potentially larger repos
            output_dir: PathBuf::from("output"),
        };

        copy::main(args)
    }).await;

    match result {
        Ok(Ok(content)) => (StatusCode::OK, content).into_response(),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)).into_response(),
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/:org/:repo", get(fetch_repo));

    Ok(router.into())
}