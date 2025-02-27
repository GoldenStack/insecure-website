use askama_axum::{IntoResponse, Response, Template};
use axum::{body::Body, http::header::CONTENT_TYPE};
use crate::{database::{db_authenticate, db_delete, db_get_verified, db_insert, db_retrieve, db_update, db_username_dump, db_verified, User, DB}, HEIGHT, HOSTNAME, WIDTH};

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
        s if check(s, ["register"]) => Register { username_err: "", password_err: "" }.into_response(),
        s if check(s, ["browse"]) => {
            let users = match db_get_verified(db) {
                Ok(users) => users,
                Err(e) => return error("could not get users", &e.to_string()).into_response(),
            };

            Browse { users }.into_response()
        },
        s if check(s, ["login", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            wrapped_db_auth(db, username, password,
                || wrapped_db_retrieve(db, username, |user| Set { user, password }))
        }
        s if check(s, ["logout", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            wrapped_db_auth(db, username, password, || Index)
        }
        s if check(s, ["register", "username", "_", "password", "_"]) => {
            let (username, password) = (s[2], s[4]);

            if username.len() + password.len() > 23 {
                let response = "read the big red message !!";
                return Register { username_err: response, password_err: response }.into_response()
            }

            if password.is_empty() {
                return Register { username_err: "", password_err: "okay, not that insecure." }.into_response()
            }

            match username.len() {
                0 => Register { username_err: "trying to register an empty username! how special.", password_err: "" }.into_response(),
                1..=2 => Register { username_err: "too short. at least 3 characters please", password_err: "" }.into_response(),
                3..=16 => match db_insert(db, username, password) {
                    Err(e) => credentials_error(&e.to_string()).into_response(),
                    Ok(true) => wrapped_db_retrieve(db, username, |user| Set { user, password }),
                    Ok(false) => Register { username_err: "that username is taken !", password_err: "" }.into_response(),
                },
                17.. => Register { username_err: "too long. at most 16 characters please", password_err: "" }.into_response(),
            }
        }
        s if check(s, ["get", "username", "_"]) => {
            let username = s[2];

            wrapped_db_retrieve(db, username, |user| Get { user })
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
                    Err(e) => error("sql error while updating db", &e.to_string()).into_response(),
                    Ok(_) => wrapped_db_retrieve(db, username, |user| Set { user, password }),
                })
            },
            s if check(s, ["admin", crate::ADMIN_PASSWORD]) => Admin.into_response(),
            s if check(s, ["admin", crate::ADMIN_PASSWORD, "verify", "_"]) => {
                match db_verified(db, s[3], true) {
                    Ok(_) => Admin.into_response(),
                    Err(e) => error("sql error", &e.to_string()).into_response()
                }
            },
            s if check(s, ["admin", crate::ADMIN_PASSWORD, "unverify", "_"]) => {
                match db_verified(db, s[3], false) {
                    Ok(_) => Admin.into_response(),
                    Err(e) => error("sql error", &e.to_string()).into_response()
                }
            },
            s if check(s, ["admin", crate::ADMIN_PASSWORD, "username", "dump"]) => {
                match db_username_dump(db) {
                    Ok(users) => Response::builder()
                        .header(CONTENT_TYPE, "text/plain; charset=utf-8;")
                        .body(Body::from(users.join("\n")))
                        .unwrap(),
                    Err(e) => error("sql error", &e.to_string()).into_response()
                }
            },
            s if check(s, ["admin", crate::ADMIN_PASSWORD, "delete", "_"]) => {
                match db_delete(db, s[3]) {
                    Ok(_) => Admin.into_response(),
                    Err(e) => error("sql error", &e.to_string()).into_response()
                }
            }
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
#[template(path = "browse.html")]
pub struct Browse {
    pub users: Vec<String>
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login;

#[derive(Template)]
#[template(path = "register.html")]
pub struct Register<'a> {
    username_err: &'a str,
    password_err: &'a str,
}

#[derive(Template)]
#[template(path = "get.html")]
pub struct Get {
    pub user: User,
}

#[derive(Template)]
#[template(path = "set.html")]
pub struct Set<'a> {
    pub user: User,
    pub password: &'a str,
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct Admin;

impl Get {
    pub fn username(&self) -> &str {
        self.user.username()
    }

    pub fn verified(&self) -> bool {
        self.user.verified()
    }
    
    pub fn checked(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.user.boxes(), *x, *y) { " checked" } else { "" }
    }
}

impl<'a> Set<'a> {
    pub fn username(&self) -> &str {
        self.user.username()
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn verified(&self) -> bool {
        self.user.verified()
    }

    pub fn checked(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.user.boxes(), *x, *y) { " checked" } else { "" }
    }

    pub fn prefix(&self, x: &u32, y: &u32) -> &'static str {
        if get_box(self.user.boxes(), *x, *y) { "uncheck" } else { "check" }
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
