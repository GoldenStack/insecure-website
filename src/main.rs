pub mod database;
pub mod response;

use std::sync::{Arc, OnceLock};

use askama_axum::Response;
use axum::{
    body::Body, extract::Host, http::{header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE}, Request}, response::IntoResponse, routing::get, Router
};
use anyhow::Result;
use database::{db_initialize, db_verified};
use dotenv_codegen::dotenv;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use response::{error, handle_query, parse_host};

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
pub const SECURE_HASHING: &str = dotenv!("secure_hashing");

pub const WIDTH: u32 = 5;
pub const HEIGHT: u32 = 5;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = SqliteConnectionManager::file(DATABASE);
    let pool = Pool::new(manager)?;
    let arc = Arc::new(pool);

    let db = &arc.get()?;

    db_initialize(db)?;

    fn html<F: IntoResponse>(f: F) -> Response {
        ([(CONTENT_TYPE, "text/html; charset=utf-8")], f).into_response()
    }

    let r = arc.clone();
    let handler = |Host(mut hostname): Host, request: Request<Body>| async move {
        println!("giga nerd request @ {}", hostname);

        // fix any sillies sent over our computing networks :)
        hostname.make_ascii_lowercase();

        let host = match parse_host(&hostname) {
            Ok(host) => host,
            Err(e) => return e,
        };

        match host {
            "assets-css" => return ([(CONTENT_TYPE, "text/css; charset=utf-8"), (ACCESS_CONTROL_ALLOW_ORIGIN, "*")], css()).into_response(),
            "assets-font" => return ([(CONTENT_TYPE, "font/woff2"), (ACCESS_CONTROL_ALLOW_ORIGIN, "*")], include_bytes!("../public/assets/Atkinson-Hyperlegible-Regular-102a.woff2")).into_response(),
            _ => {}
        };

        match r.get() {
            Ok(db) => html(handle_query(&db, host)),
            Err(e) => html(error("an error occurred while retrieving the database:", &e.to_string()))
        }
    };

    // no need for multiple handlers because of the incredible good design of our app
    let app = Router::new().route("/", get(handler));

    // port 1472 generated courtesy of random.org
    let listener = tokio::net::TcpListener::bind("127.0.0.1:1472")
        .await
        .unwrap();

    // help me
    println!("ufortuately we are listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

pub fn css() -> &'static str {
    static CSS: OnceLock<String> = OnceLock::new();
    CSS.get_or_init(|| include_str!("../public/assets/style.css")
        .replace("{{ crate::PREFIX }}", crate::PREFIX)
        .replace("{{ crate::HOSTNAME }}", crate::HOSTNAME)
    )
}
