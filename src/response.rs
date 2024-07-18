use askama_axum::{IntoResponse, Response, Template};

use crate::{database::{db_authenticate, db_insert, db_retrieve, db_update, User, DB}, parse::Query, WIDTH};

pub fn respond(query: &Query, db: &DB) -> Response {
    match query {
        Query::Index => Index.into_response(),
        Query::Login => Login.into_response(),
        Query::Register => Register.into_response(),
        Query::LoginAttempt { username, password } => wrapped_db_auth(db, username, password,
            || wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() })),
        Query::Logout { username, password } => wrapped_db_auth(db, username, password, || "Logout success!"),
        Query::RegisterAttempt { username, password } => match username.len() {
            0 => error("invalid username", "trying to register a zero-length username! how special.").into_response(),
            1..=2 => error("invalid username", "sorry, your username is too short. three characters at minimum, please").into_response(),
            3..=16 => match db_insert(db, username, password) {
                Err(e) => credentials_error(&e.to_string()).into_response(),
                Ok(true) => wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() }),
                Ok(false) => error("invalid username", "sorry, but an account with that username already exists.").into_response(),
            },
            17.. => error("invalid username", "sorry, your username is too long. sixteen characters at maximum, please").into_response(),
        },
        Query::Get { username } => wrapped_db_retrieve(db, username, |user| Get { username, boxes: user.boxes() }),
        Query::Set { x, y, checked, username, password } =>
            wrapped_db_auth(db, username, password,
                || match db_update(db, username, |data| set_box(data, *x, *y, *checked)) {
            Err(e) => credentials_error(&e.to_string()).into_response(),
            Ok(_) => wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() }),
        }),
    }
}

pub fn wrapped_db_auth<F: Fn() -> T, T: IntoResponse>(db: &DB, username: &str, password: &str, f: F) -> Response {
    match db_authenticate(db, username, password) {
        Err(e) => credentials_error(&e.to_string()).into_response(),
        Ok(false) => invalid_credentials().into_response(),
        Ok(true) => f().into_response(),
    }
}

pub fn wrapped_db_retrieve<F: Fn(User) -> T, T: IntoResponse>(db: &DB, username: &str,f: F) -> Response {
    match db_retrieve(db, username) {
        Err(e) => error(&format!("an error occurred while retrieving the data for user {}:", username), &e.to_string()).into_response(),
        Ok(None) => error("invalid username", &format!("could not find a user with the name {}!", username)).into_response(),
        Ok(Some(user)) => f(user).into_response(),
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

pub fn error<'a>(header: &'a str, description: &'a str) -> Error<'a> {
    Error {
        header, description
    }
}

pub fn credentials_error<'a>(description: &'a str) -> Error<'a> {
    Error {
        header: "an error occurred while validating your username or password: ",
        description
    }
}

pub fn invalid_credentials<'a>() -> Error<'a> {
    Error {
        header: "invalid credentials!",
        description: "unfortunately, either your username or password was incorrect.",
    }
}
