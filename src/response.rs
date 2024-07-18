use std::sync::OnceLock;

use askama::Template;

use crate::{database::{db_authenticate, db_insert, db_retrieve, db_update, DB}, parse::Query, WIDTH};

pub type Response = ([(axum::http::HeaderName, &'static str); 1], String);


pub fn respond(query: &Query, db: &DB) -> String {
    match query {
        Query::Index => index().to_string(),
        Query::Login => login().to_string(),
        Query::Register => register().to_string(),
        Query::LoginAttempt { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => credentials_error(e),
                Ok(true) => match db_retrieve(db, username) {
                    Err(e) => error(&format!("an error occurred while retrieving the data for user {}:", username), e),
                    Ok(None) => error_str("invalid username", &format!("could not find a user with the name {}!", username)),
                    Ok(Some(user)) => set(username, password, user.boxes()),
                },
                Ok(false) => invalid_credentials().to_string(),
            }
        },
        Query::Logout { username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => credentials_error(e),
                Ok(true) => "Logout success!".to_string(),
                Ok(false) => invalid_credentials().to_string(), // However, we don't care because you were just logging out.
            }
        },
        Query::RegisterAttempt { username, password } => {
            match username.len() {
                0 => error_str("invalid username", "trying to register a zero-length username! how special."),
                1..=2 => error_str("invalid username", "sorry, your username is too short. three characters at minimum, please"),
                3..=16 => match db_insert(db, username, password) {
                    Err(e) => credentials_error(e),
                    Ok(true) => match db_retrieve(db, username) {
                        Err(e) => error(&format!("an error occurred while retrieving the data for user {}:", username), e),
                        Ok(None) => error_str("invalid username", &format!("could not find a user with the name {}!", username)),
                        Ok(Some(user)) => set(username, password, user.boxes()),
                    },
                    Ok(false) => error_str("invalid username", "sorry, but an account with that username already exists."),
                },
                17.. => error_str("invalid username", "sorry, your username is too long. sixteen characters at maximum, please"),
            }
        },
        Query::Get { username } => {
            match db_retrieve(db, username) {
                Err(e) => error(&format!("an error occurred while retrieving the data for user {}:", username), e),
                Ok(None) => error_str("invalid username", &format!("could not find a user with the name {}!", username)),
                Ok(Some(user)) => get(username, user.boxes()),
            }
        },
        Query::Set { x, y, checked, username, password } => {
            match db_authenticate(db, username, password) {
                Err(e) => credentials_error(e),
                Ok(false) => invalid_credentials().to_string(),
                Ok(true) => match db_update(db, username, |data| set_box(data, *x, *y, *checked)) {
                    Err(e) => credentials_error(e),
                    Ok(_) => match db_retrieve(db, username) {
                        Err(e) => error(&format!("an error occurred while retrieving the data for user {}:", username), e),
                        Ok(None) => error_str("invalid username", &format!("could not find a user with the name {}!", username)),
                        Ok(Some(user)) => set(username, password, user.boxes()),
                    },
                },
            }
        },
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct Index;

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login;

#[derive(Template)]
#[template(path = "register.html")]
pub struct Register;

#[derive(Template)]
#[template(path = "get.html")]
pub struct Get<'a> {
    pub username: &'a str,
    pub boxes: u64,
}

#[derive(Template)]
#[template(path = "set.html")]
pub struct Set<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub boxes: u64,
}

impl<'a> Get<'a> {
    pub fn checked(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.boxes, *x, *y) { " checked" } else { "" }
    }
}

impl<'a> Set<'a> {
    pub fn checked(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.boxes, *x, *y) { " checked" } else { "" }
    }

    pub fn prefix(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.boxes, *x, *y) { "uncheck" } else { "check" }
    }
}

pub fn get_box(boxes: u64, x: u32, y: u32) -> bool {
    (boxes >> (y * WIDTH + x)) & 1 == 1
}

pub fn set_box(boxes: u64, x: u32, y: u32, checked: bool) -> u64 {
    let mask = 0b1 << (y * WIDTH + x);
    if checked { boxes | mask } else { boxes & !mask }
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct Error<'a> {
    pub header: &'a str,
    pub description: &'a str,
}

pub fn index() -> &'static str {
    static INDEX: OnceLock<String> = OnceLock::new();
    INDEX.get_or_init(|| Index.render().unwrap())
}

pub fn login() -> &'static str {
    static LOGIN: OnceLock<String> = OnceLock::new();
    LOGIN.get_or_init(|| Login.render().unwrap())
}

pub fn register() -> &'static str {
    static REGISTER: OnceLock<String> = OnceLock::new();
    REGISTER.get_or_init(|| Register.render().unwrap())
}

pub fn error_str<'a>(header: &'a str, description: &'a str) -> String {
    Error {
        header, description
    }.render().unwrap()
}

pub fn error<'a>(message: &'a str, error: anyhow::Error) -> String {
    error_str(message, &error.to_string())
}

pub fn credentials_error(e: anyhow::Error) -> String {
    error("an error occurred while validating your username or password: ", e)
}

pub fn invalid_credentials() -> &'static str {
    static CREDS: OnceLock<String> = OnceLock::new();
    CREDS.get_or_init(|| {
        Error {
            header: "invalid credentials!",
            description: "unfortunately, either your username or password was incorrect.",
        }.render().unwrap()
    })
}

pub fn get<'a>(username: &'a str, boxes: u64) -> String {
    Get {
        username, boxes
    }.render().unwrap()
}

pub fn set<'a>(username: &'a str, password: &'a str, boxes: u64) -> String {
    Set {
        username, password, boxes
    }.render().unwrap()
}