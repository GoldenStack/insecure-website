use axum::{
    body::Body, extract::{Host, Request}, response::Html, routing::get, Router
};
use anyhow::Result;
use thiserror::Error;

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

#[derive(Debug)]
enum Query<'a> {
    Login { username: &'a str, password: &'a str },
    Logout { username: &'a str, password: &'a str },
    Register { username: &'a str, password: &'a str },
    Get { username: &'a str },
    Set { x: u8, y: u8, checked: bool, username: &'a str, password: &'a str }
}

#[derive(Error, Debug)]
enum CatastrophicFailure {
    #[error("Bad hostname. Please try again never.")]
    BadHostname,

    #[error("No subdomain.")]
    NoSubdomain,

    #[error("Unknown subdomain query {0}")]
    UnknownSubdomainQuery(String),

    #[error("Invalid length of query.")]
    InvalidQueryLength,

    #[error("Expected text {text} at position {position}.")]
    ExpectedQueryText {
        text: String,
        position: usize
    },

    #[error("Illegal character in query.")]
    IllegalCharacter
}

const HOSTNAME: &str = "secure-website.localhost:1472";

#[tokio::main]
async fn main() -> Result<()> {
    // no need for multiple handlers because of the incrediblly good design of our app
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

// sidenote: we can just completely ignore CORS because HEAD requests give us all the information we need

async fn handler(Host(hostname): Host, request: Request<Body>) -> Html<String> {
    println!("giga nerd request @ {}", hostname);

    let query = match parse_host(&hostname).and_then(parse_query) {
        Ok(q) => q,
        Err(e) => return Html(format!("Error: {:?}", e))
    };

    println!("parsed: {:?}", query);

    Html(format!("<h1>{:?}</h1>", query))
}

fn parse_host<'a>(host: &'a str) -> Result<&'a str> {

    // Parse out hostname
    if !host.ends_with(HOSTNAME) {
        return Err(CatastrophicFailure::BadHostname.into());
    }

    let host = &host[0..host.len()-HOSTNAME.len()];

    if !host.ends_with(".") {
        return Err(CatastrophicFailure::NoSubdomain.into());
    }

    let mut host = &host[0..host.len()-1];

    if let Some(index) = host.rfind(".") {
        host = &host[index+1..];
    }

    if !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(CatastrophicFailure::IllegalCharacter.into())
    }

    Ok(host)
}

fn parse_query<'a>(string: &'a str) -> Result<Query<'a>> {

    // let me know if i can not allocate here somehow
    let sections = string.split("-").collect::<Vec<_>>();

    let Some(first) = sections.get(0) else {
        return Err(CatastrophicFailure::InvalidQueryLength.into());
    };

    match *first {
        "login" => {
            if let Err(e) = require_params(&sections, ["login", "username", "_", "password", "_"]) {
                return Err(e);
            }
            Ok(Query::Login { username: sections[2], password: sections[4] })
        }
        "logout" => {
            if let Err(e) = require_params(&sections, ["logout", "username", "_", "password", "_"]) {
                return Err(e);
            }
            Ok(Query::Logout { username: sections[2], password: sections[4] })
        }
        "register" => {
            if let Err(e) = require_params(&sections, ["register", "username", "_", "password", "_"]) {
                return Err(e);
            }
            Ok(Query::Register { username: sections[2], password: sections[4] })
        }
        "get" => {
            if let Err(e) = require_params(&sections, ["get", "username", "_"]) {
                return Err(e);
            }
            Ok(Query::Get { username: sections[2] })
        }
        "set" => {
            if let Err(e) = require_params(&sections, ["set", "checkbox", "x", "_", "y", "_", "to", "_", "username", "_", "password", "_"]) {
                return Err(e);
            }

            let Ok(x) = sections[3].parse() else {
                return Err(CatastrophicFailure::IllegalCharacter.into());
            };

            let Ok(y) = sections[5].parse() else {
                return Err(CatastrophicFailure::IllegalCharacter.into());
            };

            let checked = match sections[7] {
                "checked" => true,
                "unchecked" => false,
                _ => return Err(CatastrophicFailure::IllegalCharacter.into())
            };

            Ok(Query::Set { x, y, checked, username: sections[9], password: sections[11] })
        }
        _ => Err(CatastrophicFailure::UnknownSubdomainQuery(first.to_string()).into())
    }
}

fn require_params<const N: usize>(query: &Vec<&str>, params: [&str; N]) -> Result<()> {
    
    if query.len() != params.len() {
        return Err(CatastrophicFailure::InvalidQueryLength.into());
    }

    for (index, str) in params.iter().enumerate() {
        if str == &"_" { continue; }

        if &query[index] != str {
            return Err(CatastrophicFailure::ExpectedQueryText { text: str.to_string(), position: index }.into())
        }
    }

    Ok(())
}