use axum::{
    body::Body, extract::{Host, Request}, response::Html, routing::get, Router
};
use anyhow::Result;

// request types:

// logging in: `login-username-<USERNAME>-password-<PASSWORD>.https.goldenstack.net`
//  - serves no real purpose other than to give the client immediate feedback as to whether or not they've logged in

// logging out: `logout-username-<USERNAME>-password-<PASSWORD>.https.goldenstack.net`
//  - serves actual no real purpose except to be less secure

// register: `register-username-<USERNAME>-password-<PASSWORD>.https.goldenstack.net`
//  - incredible

// getting data: `get-username-<USERNAME>.https.goldenstack.net`
//  - just returns the data. i dunno what you'd expect

// setting data: `set-checkbox-x-<X>-y-<Y>-to-checked-username-<USERNAME>-password-<PASSWORD>.https.goldenstack.net``
//  - you might wonder: why do we login externally if you also login here?
//  - well the reason why we login here is because tokens are too hard
//  - and the reason we log in in other places is because we can
//  - theoretically the server itself is secure. you do need the password. it's
//  - just that it's rather easy to acquire it

// for some reason web frameworks support placeholders in example.com/PLACEHOLDER
// but not PLACEHOLDER.example.com. i wonder why.

#[tokio::main]
async fn main() -> Result<()> {
    // no need for multiple handlers because of the incrediblly good design of our app
    let app = Router::new().route("/", get(handler));

    // port 1472 generated courtesy of random.org
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1472")
        .await
        .unwrap();

    println!("ufortuately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// we can just completely ignore CORS because HEAD requests give us all the information we need

async fn handler(Host(hostname): Host, request: Request<Body>) -> Html<String> {
    println!("giga nerd request @ {}", hostname);
    Html(format!("<h1>{}</h1>", hostname))
}