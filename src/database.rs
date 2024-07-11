use anyhow::{anyhow, Result};
use argon2::Argon2;
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ErrorCode;

type DB = PooledConnection<SqliteConnectionManager>;

#[derive(Debug)]
pub struct User {
    id: u64,
    username: String,
    password: [u8; 32],
    salt: [u8; 8],
    boxes: u64,
}

impl User {
    pub fn boxes(&self) -> u64 {
        self.boxes
    }
}

pub fn db_initialize(db: &DB) -> Result<()> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id       INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            password BLOB,
            salt     BLOB,
            boxes    BLOB,

            UNIQUE(username) ON CONFLICT IGNORE
        )", ())?;

    db.execute("CREATE UNIQUE INDEX IF NOT EXISTS user_index ON users(username)", ())?;

    Ok(())
}

pub fn db_retrieve(db: &DB, username: &str) -> Result<Option<User>> {
    let mut get = db.prepare("SELECT id, username, password, salt, boxes FROM users WHERE username = ?1")?;

    let mut users = get.query_map([username], |row| {
        Ok(User {
            id: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?,
            salt: row.get(3)?,
            boxes: row.get(4)?
        })
    })?;

    return Ok(if let Some(result) = users.next() {
        Some(result?)
    } else {
        None
    })
}

pub fn db_insert(db: &DB, username: &str, password: &str) -> Result<bool> {

    if db_retrieve(db, username)?.is_some() {
        return Ok(false);
    }

    let (password, salt) = hash(password)?;

    db.execute(
        "INSERT INTO users (username, password, salt, boxes) VALUES (?1, ?2, ?3, ?4)",
        (username, password, salt, 0),
    ).map(|_| true)
    .or_else(|e| {
        if let rusqlite::Error::SqliteFailure(error, _) = e {
            if error.code == ErrorCode::ConstraintViolation {
                return Ok(false);
            }
        }
    
        return Err(anyhow!("SQLite error: {}", e));
    })
}

pub fn db_update<F: Fn(u64) -> u64>(db: &DB, username: &str, f: F) -> Result<bool> {

    let Some(user) = db_retrieve(db, username)? else {
        return Ok(false);
    };

    db.execute(
        "UPDATE users SET boxes = ?1 WHERE id = ?2", (f(user.boxes), user.id)
    )?;

    Ok(true)
}

pub fn db_authenticate(db: &DB, username: &str, password: &str) -> Result<bool> {
    let Some(user) = db_retrieve(db, username)? else {
        return Ok(false);
    };

    let result = hash_with_salt(password, user.salt)? == user.password;

    Ok(result)
}

fn hash(password: &str) -> Result<([u8; 32], [u8; 8])> {
    let mut salt = [0u8; 8];

    getrandom::getrandom(&mut salt)
        .map_err(|e| anyhow!("Could not get OS random: {}", e))?;

    let hash = hash_with_salt(password, salt)?;

    Ok((hash, salt))
}

fn hash_with_salt(password: &str, salt: [u8; 8]) -> Result<[u8; 32]> {
    let mut output = [0u8; 32];
    Argon2::default().hash_password_into(password.as_bytes(), &salt, &mut output)
        .map_err(|e| anyhow!("Could not hash password: {}", e))?;

    Ok(output)
}