use axum::response::Html;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;

use crate::{database::{db_authenticate, db_insert, db_retrieve, db_update}, parse::Query};

type DB = PooledConnection<SqliteConnectionManager>;

pub fn respond(query: &Query, db: &DB) -> Html<String> {
    match query {
        Query::Login { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => Html(format!("An error occurred while validating your username or password: {}", e)),
                Ok(true) => Html("Login success!".to_string()),
                Ok(false) => Html("Invalid username or password!".to_string()),
            }
        },
        Query::Logout { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => Html(format!("An error occurred while validating your username or password: {}", e)),
                Ok(true) => Html("Logout success!".to_string()),
                Ok(false) => Html("Invalid username or password! However, we don't care because you were just logging out.".to_string()),
            }
        },
        Query::Register { username, password } => {
            match db_insert(db, username, password) {
                Err(e) => Html(format!("An error occurred while validating your username or password: {}", e)),
                Ok(true) => Html(format!("Your account was registered!")),
                Ok(false) => Html(format!("An account with the username '{}' already exists!", username)),
            }
        },
        Query::Get { username } => {
            match db_retrieve(db, username) {
                Err(e) => Html(format!("An error occurred while retrieving the data for user {}: {}", username, e)),
                Ok(None) => Html(format!("Could not find a user with the name {}!", username)),
                Ok(Some(user)) => Html(format!("Data for username {}:\n{:025b}", username, user.boxes())),
            }
        },
        Query::Set { x, y, checked, username, password } => {
            let mask = 0b1 << (y * 5 + x);
            match db_authenticate(db, username, password) {
                Err(e) => Html(format!("An error occurred while validating your username or password: {}", e)),
                Ok(false) => Html("Invalid username or password!".to_string()),
                Ok(true) => match db_update(db, username, |data| if *checked { data | mask } else { data & mask }) {
                    Err(e) => Html(format!("An error occurred while validating your username or password: {}", e)),
                    Ok(_) => Html(format!("({}, {}) set to {}!", x, y, checked)),
                },
            }
        },
    }
}