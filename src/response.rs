use askama::Template;
use axum::response::Html;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

use crate::{database::{db_authenticate, db_insert, db_retrieve, db_update}, parse::Query};

type DB = PooledConnection<SqliteConnectionManager>;

pub fn respond_html(query: &Query, db: &DB) -> Html<String> {
    Html(respond(query, db))
}

pub fn respond(query: &Query, db: &DB) -> String {
    match query {
        Query::Index => {
            Index.render().unwrap()
        },
        Query::Login { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => Error { header: "An error occurred while validating your username or password: ", description: &e.to_string()}.render().unwrap(),
                Ok(true) => "Login success!".to_string(),
                Ok(false) => INVALID_CREDENTIALS.render().unwrap(),
            }
        },
        Query::Logout { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => Error { header: "An error occurred while validating your username or password: ", description: &e.to_string()}.render().unwrap(),
                Ok(true) => "Logout success!".to_string(),
                Ok(false) => INVALID_CREDENTIALS.render().unwrap(), // However, we don't care because you were just logging out.
            }
        },
        Query::Register { username, password } => {
            match username.len() {
                0 => "Trying to register a zero-length username! How special.".to_string(),
                1..=2 => "Sorry, your username is too short. Three characters at minimum, please.".to_string(),
                3..=16 => match db_insert(db, username, password) {
                    Err(e) => format!("An error occurred while validating your username or password: {}", e),
                    Ok(true) => format!("Your account was registered!"),
                    Ok(false) => format!("An account with the username '{}' already exists!", username),
                },
                17.. => "Sorry, your username is too long. Three characters at minimum, please.".to_string(),
            }
        },
        Query::Get { username } => {
            match db_retrieve(db, username) {
                Err(e) => Error { header: &format!("An error occurred while retrieving the data for user {}:", username), description: &e.to_string()}.render().unwrap(),
                Ok(None) => format!("Could not find a user with the name {}!", username),
                Ok(Some(user)) => format!("Data for username {}:\n{:025b}", username, user.boxes()),
            }
        },
        Query::Set { x, y, checked, username, password } => {
            let mask = 0b1 << (y * 5 + x);
            match db_authenticate(db, username, password) {
                Err(e) => Error { header: "An error occurred while validating your username or password: ", description: &e.to_string()}.render().unwrap(),
                Ok(false) => INVALID_CREDENTIALS.render().unwrap(),
                Ok(true) => match db_update(db, username, |data| if *checked { data | mask } else { data & !mask }) {
                    Err(e) => Error { header: "An error occurred while validating your username or password: ", description: &e.to_string()}.render().unwrap(),
                    Ok(_) => format!("({}, {}) set to {}!", x, y, checked),
                },
            }
        },
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

#[derive(Template)]
#[template(path = "error.html")]
struct Error<'a> {
    header: &'a str,
    description: &'a str,
}

const INVALID_CREDENTIALS: Error = Error {
    header: "invalid credentials!",
    description: "unfortunately, either your username or password was incorrect.",
};