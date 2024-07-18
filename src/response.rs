use askama_axum::{IntoResponse, Response, Template};
use crate::{database::{db_authenticate, db_insert, db_retrieve, db_update, User, DB}, HEIGHT, HOSTNAME, WIDTH};

/// Parses the subdomain (all of our data) out of a hostname.
/// This uses the [HOSTNAME] constant, so make sure to update that when
/// necessary!
pub fn parse_host<'a>(host: &'a str) -> Result<&'a str, Response> {

    if !host.ends_with(HOSTNAME) {
        return Err(error("bad hostname", "please try again never").into_response());
    }

    let mut host = &host[0..host.len()-HOSTNAME.len()];

    if let Some(index) = host.rfind(".") {
        host = &host[index+1..];
    }

    if !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(error("bad hostname", "you know what you did. nice illegal character").into_response());
    }

    Ok(host)
}

/// Handles a query, returning a response.
/// Example queries:
/// check-2-2-username-goldenstack-password-dosbox074
/// get-username-goldenstack
pub fn handle_query<'a>(db: &DB, string: &'a str) -> Response {

    // let me know if i can not allocate here somehow
    match &string.split("-").collect::<Vec<_>>() {
        s if check(s, ["index"]) => Index.into_response(),
        s if check(s, ["login"]) => Login.into_response(),
        s if check(s, ["register"]) => Register.into_response(),
        s if check(s, ["login", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            wrapped_db_auth(db, username, password,
                || wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() }))
        }
        s if check(s, ["logout", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            wrapped_db_auth(db, username, password, || "Logout success!")
        }
        s if check(s, ["register", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            match username.len() {
                0 => error("invalid username", "trying to register a zero-length username! how special.").into_response(),
                1..=2 => error("invalid username", "sorry, your username is too short. three characters at minimum, please").into_response(),
                3..=16 => match db_insert(db, username, password) {
                    Err(e) => credentials_error(&e.to_string()).into_response(),
                    Ok(true) => wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() }),
                    Ok(false) => error("invalid username", "sorry, but an account with that username already exists.").into_response(),
                },
                17.. => error("invalid username", "sorry, your username is too long. sixteen characters at maximum, please").into_response(),
            }
        }
        s if check(s, ["get", "username", "_"]) => {
            let username = s[2];

            wrapped_db_retrieve(db, username, |user| Get { username, boxes: user.boxes() })
        }
        s if check(s, ["check", "_", "_", "username", "_", "password", "_"])
            || check(s, ["uncheck", "_", "_", "username", "_", "password", "_"]) => {

                let Some(x) = s.get(1).and_then(|&s| s.parse::<u32>().ok().filter(|&x| x < WIDTH)) else {
                    return error("bad subdomain query", "x was either invalid or out of bounds, sorry!").into_response()
                };

                let Some(y) = s.get(2).and_then(|&s| s.parse::<u32>().ok().filter(|&y| y < HEIGHT)) else {
                    return error("bad subdomain query", "y was either invalid or out of bounds, sorry!").into_response()
                };

                let Some(checked) = s.get(0).and_then(|&s| match s {
                    "check" => Some(true),
                    "uncheck" => Some(false),
                    _ => None
                }) else {
                    return error("bad subdomain query", "expected 'check' or 'uncheck' as a subdomain query, but found something else").into_response()
                };

                let (username, password) = (s[4], s[6]);

                wrapped_db_auth(db, username, password,
                        || match db_update(db, username, |data| set_box(data, x, y, checked)) {
                    Err(e) => credentials_error(&e.to_string()).into_response(),
                    Ok(_) => wrapped_db_retrieve(db, username, |user| Set { username, password, boxes: user.boxes() }),
                })
            },
            _ => error("unknown subdomain query", "unfortunately your subdomain query isn't recognized, so i have no idea what you're trying to do.").into_response()
    }

}

fn check<const N: usize>(query: &Vec<&str>, params: [&str; N]) -> bool {
    query.len() == N && params.iter()
        .enumerate()
        .filter(|(_, &s)| s != "_")
        .all(|(segment, &str)| query[segment] == str)
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
