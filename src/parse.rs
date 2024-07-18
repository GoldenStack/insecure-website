use thiserror::Error;

use crate::{HEIGHT, HOSTNAME, WIDTH};

/// A simple query sent to this server
/// There actually isn't anything more in a query. It's pretty simple.
#[derive(Debug)]
pub enum Query<'a> {
    Index,
    Login,
    Register,
    LoginAttempt { username: &'a str, password: &'a str },
    RegisterAttempt { username: &'a str, password: &'a str },
    Logout { username: &'a str, password: &'a str },
    Get { username: &'a str },
    Set { x: u32, y: u32, checked: bool, username: &'a str, password: &'a str }
}

/// Indicates a request could not be parsed for some reason (i.e., a
/// catastrophic failure).
#[derive(Error, Debug)]
pub enum CatastrophicFailure {
    #[error("Bad hostname (expected {HOSTNAME}). Please try again never.")]
    BadHostname,

    #[error("Unknown subdomain query. insecure-website transmits all information to the server through the subdomain.")]
    UnknownSubdomainQuery,

    #[error("Illegal character in query. You know what you did.")]
    IllegalCharacter,

    #[error("Could not parse segment {segment} in query {query}.")]
    Unparseable {
        segment: usize,
        query: String,
    }
}

/// Parses the subdomain (all of our data) out of a hostname.
/// This uses the [HOSTNAME] constant, so make sure to update that when
/// necessary!
pub fn parse_host<'a>(host: &'a str) -> Result<&'a str, CatastrophicFailure> {

    if !host.ends_with(HOSTNAME) {
        return Err(CatastrophicFailure::BadHostname);
    }

    let mut host = &host[0..host.len()-HOSTNAME.len()];

    if let Some(index) = host.rfind(".") {
        host = &host[index+1..];
    }

    if !host.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(CatastrophicFailure::IllegalCharacter);
    }

    Ok(host)
}

/// Parses a query string into a query struct.
/// Example queries:
/// check-2-2-username-goldenstack-password-dosbox074
/// get-username-goldenstack
pub fn parse_query<'a>(string: &'a str) -> Result<Query<'a>, CatastrophicFailure> {

    // let me know if i can not allocate here somehow
    match &string.split("-").collect::<Vec<_>>() {
        s if check(s, ["index"]) => Ok(Query::Index),
        s if check(s, ["login"]) => Ok(Query::Login),
        s if check(s, ["register"]) => Ok(Query::Register),
        s if check(s, ["login", "username", "_", "password", "_"]) =>
            Ok(Query::LoginAttempt { username: s[2], password: s[4] }),
        s if check(s, ["logout", "username", "_", "password", "_"]) =>
            Ok(Query::Logout { username: s[2], password: s[4] }),
        s if check(s, ["register", "username", "_", "password", "_"]) =>
            Ok(Query::RegisterAttempt { username: s[2], password: s[4] }),
        s if check(s, ["get", "username", "_"]) => Ok(Query::Get { username: s[2] }),
        s if check(s, ["check", "_", "_", "username", "_", "password", "_"])
            || check(s, ["uncheck", "_", "_", "username", "_", "password", "_"]) => {
                let x = require_parse(&s, 1,
                    |x| x.parse::<u32>().ok().filter(|&x| x < WIDTH))?;
                let y = require_parse(&s, 2,
                    |y| y.parse::<u32>().ok().filter(|&y| y < HEIGHT))?;

                let checked = require_parse(&s, 0, |s| match s {
                    "check" => Some(true),
                    "uncheck" => Some(false),
                    _ => None
                })?;

                Ok(Query::Set { x, y, checked, username: s[4], password: s[6] })
            },
            _ => Err(CatastrophicFailure::UnknownSubdomainQuery)
    }

}

fn require_parse<T, F: Fn(&str) -> Option<T>>(query: &Vec<&str>, segment: usize, parser: F) -> Result<T, CatastrophicFailure> {
    let string = query.get(segment).unwrap(); // we've already passed it through check.

    parser(string).ok_or_else(|| {
        CatastrophicFailure::Unparseable {
            segment,
            query: query.first().unwrap().to_string(),
        }
    })
}

fn check<const N: usize>(query: &Vec<&str>, params: [&str; N]) -> bool {
    query.len() == N && params.iter()
        .enumerate()
        .filter(|(_, &s)| s != "_")
        .all(|(segment, &str)| query[segment] == str)
}