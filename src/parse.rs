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
    Set { x: u8, y: u8, checked: bool, username: &'a str, password: &'a str }
}

/// Indicates a request could not be parsed for some reason (i.e., a
/// catastrophic failure).
#[derive(Error, Debug)]
pub enum CatastrophicFailure {
    #[error("Bad hostname (expected {HOSTNAME}). Please try again never.")]
    BadHostname,

    #[error("No subdomain. secure-website transmits all information to the server through the subdomain.")]
    NoSubdomain,

    #[error("Unknown subdomain query {0}.")]
    UnknownSubdomainQuery(String),

    #[error("Expected {count} segments for query {query}")]
    InvalidQueryLength {
        count: usize,
        query: String,
    },

    #[error("Expected text {text} at segment {segment} in query {query}.")]
    ExpectedQueryText {
        text: String,
        segment: usize,
        query: String,
    },

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
/// set-checkbox-x-5-y-2-to-unchecked-username-goldenstack-password-dosbox074
/// get-username-goldenstack
pub fn parse_query<'a>(string: &'a str) -> Result<Query<'a>, CatastrophicFailure> {

    // let me know if i can not allocate here somehow
    let sections = string.split("-").collect::<Vec<_>>();

    let Some(first) = sections.get(0) else {
        return Err(CatastrophicFailure::NoSubdomain);
    };

    match *first {
        "index" => {
            require_params(&sections, ["index"])?;
            Ok(Query::Index)
        }
        "login" => {
            if require_params(&sections, ["login"]).is_ok() {
                return Ok(Query::Login);
            }

            require_params(&sections, ["login", "username", "_", "password", "_"])?;
            Ok(Query::LoginAttempt { username: sections[2], password: sections[4] })
        }
        "logout" => {
            require_params(&sections, ["logout", "username", "_", "password", "_"])?;
            Ok(Query::Logout { username: sections[2], password: sections[4] })
        }
        "register" => {
            if require_params(&sections, ["register"]).is_ok() {
                return Ok(Query::Register);
            }

            require_params(&sections, ["register", "username", "_", "password", "_"])?;
            Ok(Query::RegisterAttempt { username: sections[2], password: sections[4] })
        }
        "get" => {
            require_params(&sections, ["get", "username", "_"])?;
            Ok(Query::Get { username: sections[2] })
        }
        "set" => {
            require_params(&sections, ["set", "checkbox", "x", "_", "y", "_", "to", "_", "username", "_", "password", "_"])?;
            
            let x = require_parse(&sections, 3,
                |x| x.parse::<u8>().ok().filter(|&x| x < WIDTH))?;
            let y = require_parse(&sections, 5,
                |y| y.parse::<u8>().ok().filter(|&y| y < HEIGHT))?;

            let checked = require_parse(&sections, 7, |s| match s {
                "checked" => Some(true),
                "unchecked" => Some(false),
                _ => None
            })?;

            Ok(Query::Set { x, y, checked, username: sections[9], password: sections[11] })
        }
        _ => Err(CatastrophicFailure::UnknownSubdomainQuery(first.to_string()))
    }
}

fn require_parse<T, F: Fn(&str) -> Option<T>>(query: &Vec<&str>, segment: usize, parser: F) -> Result<T, CatastrophicFailure>{
    let string = query.get(segment).unwrap(); // we've already passed it through require_params.

    parser(string).ok_or_else(|| {
        CatastrophicFailure::Unparseable {
            segment,
            query: query.first().unwrap().to_string(),
        }
    })
}

fn require_params<const N: usize>(query: &Vec<&str>, params: [&str; N]) -> Result<(), CatastrophicFailure> {
    
    let Some(&first) = query.first() else {
        return Err(CatastrophicFailure::NoSubdomain);
    };

    if query.len() != params.len() {
        return Err(CatastrophicFailure::InvalidQueryLength {
            count: params.len(),
            query: first.into()
        });
    }

    for (segment, &str) in params.iter().enumerate().filter(|(_, &s)| s != "_") {
        if query[segment] != str {
            return Err(CatastrophicFailure::ExpectedQueryText {
                text: str.into(),
                segment,
                query: first.into(),
            })
        }
    }

    Ok(())
}