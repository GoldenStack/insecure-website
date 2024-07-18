pub mod parse;
pub mod database;
pub mod response;

use std::sync::Arc;

use askama::Template;
use axum::{
    body::Body, extract::Host, http::{header::CONTENT_TYPE, Request}, routing::get, Router
};
use anyhow::Result;
use database::db_initialize;
use dotenv_codegen::dotenv;
use parse::{parse_host, parse_query};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use response::respond;
use crate::response::Error;

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

pub const PREFIX: &str = dotenv!("prefix");
pub const HOSTNAME: &str = dotenv!("hostname");
pub const DATABASE: &str = dotenv!("database");

pub const WIDTH: u32 = 5;
pub const HEIGHT: u32 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = SqliteConnectionManager::file(DATABASE);
    let pool = Pool::new(manager)?;
    let arc = Arc::new(pool);

    let db = &arc.get()?;

    db_initialize(db)?;

    let r = arc.clone();
    let handler = |Host(mut hostname): Host, request: Request<Body>| async move {
        println!("giga nerd request @ {}", hostname);

        // fix any sillies sent over our computing networks :)
        hostname.make_ascii_lowercase();
    
        let query = match parse_host(&hostname).and_then(parse_query) {
            Ok(q) => q,
            Err(e) => return ([(CONTENT_TYPE, "text/html; charset=utf-8")], format!("{:?}", e))
        };

        match r.get() {
            Ok(db) => ([(CONTENT_TYPE, "text/html; charset=utf-8")], respond(&query, &db)),
            Err(e) => ([(CONTENT_TYPE, "text/html; charset=utf-8")], Error { header: "An error occurred while retrieving the database:", description: &e.to_string()}.render().unwrap())
        }
    };

    // no need for multiple handlers because of the incredible good design of our app
    let app = Router::new().route("/", get(handler))
        .route("/assets/style.css", get(|| async move {
            ([(CONTENT_TYPE, "text/css; charset=utf-8")], include_bytes!("../public/assets/style.css"))
        }))
        .route("/assets/fonts/Atkinson-Hyperlegible-Regular-102a.woff2", get(|| async move {
            ([(CONTENT_TYPE, "font/woff2")], include_bytes!("../public/assets/fonts/Atkinson-Hyperlegible-Regular-102a.woff2"))
        }));

    // port 1472 generated courtesy of random.org
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1472")
        .await
        .unwrap();

    // help me
    println!("ufortuately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// sidenote: we can just completely ignore CORS because HEAD requests give us all the information we need
