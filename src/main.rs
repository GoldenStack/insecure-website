use axum::{
    body::Body, extract::{Host, Request}, response::Html, routing::get, Router
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // no need for multiple handlers because of the incrediblly good design of our app
    let app = Router::new().route("/", get(handler));

    // port 1472 generated courtesy of random.org
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1472")
        .await
        .unwrap();

    println!("ufortunately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// we do not need to handle CORS because HEAD requests are enough for us!

async fn handler(Host(hostname): Host, request: Request<Body>) -> Html<String> {
    println!("giga nerd request @ {}", hostname);
    Html(format!("<h1>{}</h1>", hostname))
}