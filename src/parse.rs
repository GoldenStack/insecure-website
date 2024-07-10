use thiserror::Error;

use crate::{HEIGHT, HOSTNAME, WIDTH};

#[derive(Debug)]
pub enum Query<'a> {
    Login { username: &'a str, password: &'a str },
    Logout { username: &'a str, password: &'a str },
    Register { username: &'a str, password: &'a str },
    Get { username: &'a str },
    Set { x: u8, y: u8, checked: bool, username: &'a str, password: &'a str }
}

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

    #[error("Illegal character at segment {segment} in query {query}.")]
    IllegalCharacter {
        segment: usize,
        query: String,
    }
}

pub fn parse_host<'a>(host: &'a str) -> Result<&'a str, CatastrophicFailure> {

    if !host.ends_with(HOSTNAME) {
        return Err(CatastrophicFailure::BadHostname);
    }

    let host = &host[0..host.len()-HOSTNAME.len()];

    if !host.ends_with(".") {
        return Err(CatastrophicFailure::NoSubdomain);
    }

    let mut host = &host[0..host.len()-1];

    if let Some(index) = host.rfind(".") {
        host = &host[index+1..];
    }

    Ok(host)
}

pub fn parse_query<'a>(string: &'a str) -> Result<Query<'a>, CatastrophicFailure> {


    // let me know if i can not allocate here somehow
    let sections = string.split("-").collect::<Vec<_>>();

    let Some(first) = sections.get(0) else {
        return Err(CatastrophicFailure::NoSubdomain);
    };

    match *first {
        "login" => {
            require_params(&sections, ["login", "username", "_", "password", "_"])?;
            Ok(Query::Login { username: sections[2], password: sections[4] })
        }
        "logout" => {
            require_params(&sections, ["logout", "username", "_", "password", "_"])?;
            Ok(Query::Logout { username: sections[2], password: sections[4] })
        }
        "register" => {
            require_params(&sections, ["register", "username", "_", "password", "_"])?;
            Ok(Query::Register { username: sections[2], password: sections[4] })
        }
        "get" => {
            require_params(&sections, ["get", "username", "_"])?;
            Ok(Query::Get { username: sections[2] })
        }
        "set" => {
            require_params(&sections, ["set", "checkbox", "x", "_", "y", "_", "to", "_", "username", "_", "password", "_"])?;
            
            let x = sections[3].parse().ok().filter(|&x| x < WIDTH).ok_or_else(|| {
                CatastrophicFailure::IllegalCharacter {
                    segment: 3,
                    query: "set".into(),
                }
            })?;

            let y = sections[5].parse().ok().filter(|&y| y < HEIGHT).ok_or_else(|| {
                CatastrophicFailure::IllegalCharacter {
                    segment: 3,
                    query: "set".into(),
                }
            })?;

            let checked = match sections[7] {
                "checked" => true,
                "unchecked" => false,
                _ => return Err(CatastrophicFailure::IllegalCharacter {
                    segment: 7,
                    query: "set".into(),
                })
            };

            Ok(Query::Set { x, y, checked, username: sections[9], password: sections[11] })
        }
        _ => Err(CatastrophicFailure::UnknownSubdomainQuery(first.to_string()))
    }
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

        if !str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(CatastrophicFailure::IllegalCharacter {
                segment,
                query: first.into()
            })
        }

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