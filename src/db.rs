use std::collections::HashMap;

use rusqlite::{params, Connection};
use crate::models::{GameStatus, User, UserRequest};
use crate::errors::DatabaseError;
use crate::token::create_token;

/// Initializes a connection to the database
/// ### Arguments
/// * `in_memory` - A boolean indicating whether to use an in-memory database.\
/// 
/// In-memory databases are useful for testing and development as they are destroyed when the program exits.
pub fn initialize_connection(in_memory: bool) -> rusqlite::Connection {
    if in_memory {
        rusqlite::Connection::open_in_memory().expect("Failed to open in-memory database")
    } else {
        rusqlite::Connection::open("database.sqlite").expect("Failed to open database")
    }
}

/// Retrieves a user from the database or creates a new user (IN-MEMORY) if it does not exist.
/// ### Arguments
/// * `conn` - A reference to the database connection
/// * `request` - A `UserRequest` enum indicating the type of request
/// ### Returns
/// A `User` struct
/// ### Errors
/// Returns a `DatabaseError` if the user does not exist or if there is an issue with the database.
/// ### IMPORTANT
/// This **does not** insert a new user to the database. It only retrieves or creates a user in-memory.
/// A user must be updated with `update_user_game_status` to be inserted into the database.
pub fn get_user(conn: &Connection, request: UserRequest) -> Result<User, DatabaseError> {
    match request {
        UserRequest::Tokens {
            stale_token,
            fresh_token,
        } => {
            let mut stmt = conn.prepare(
                "SELECT stale_token, fresh_token, game_status FROM users WHERE stale_token = ? AND fresh_token = ?",
            ).map_err(DatabaseError::FromSQLError)?;

            let mut rows = stmt.query(params![stale_token, fresh_token])
                .map_err(DatabaseError::FromSQLError)?;

            if let Some(row) = rows.next().map_err(DatabaseError::FromSQLError)? {
                let game_status_json: String = row.get(2).map_err(DatabaseError::FromSQLError)?;
                let game_status_map: HashMap<String, GameStatus> =
                    serde_json::from_str(&game_status_json)
                        .map_err(|e| DatabaseError::GameStatusParseError(e.to_string()))?;

                Ok(User {
                    stale_token: row.get(0).map_err(DatabaseError::FromSQLError)?,
                    fresh_token: Some(row.get(1).map_err(DatabaseError::FromSQLError)?),
                    game_status: game_status_map,
                })
            } else {
                Err(DatabaseError::FromSQLError(
                    rusqlite::Error::QueryReturnedNoRows,
                ))
            }
        }
        UserRequest::NewUser => {
            let new_stale_token = create_token();

            Ok(User {
                stale_token: new_stale_token,
                fresh_token: None,
                game_status: HashMap::new(),
            })
        }
    }
}

pub fn update_user_game_status(
    conn: &Connection,
    stale_token: &str,
    new_game_status: &GameStatus,
) -> Result<String, DatabaseError> {
    let new_fresh_token = create_token();

    let key = new_game_status.to_string();

    // Retrieve the existing game status from the database
    let mut stmt = conn.prepare("SELECT game_status FROM users WHERE stale_token = ?1")
        .map_err(DatabaseError::FromSQLError)?;

    let mut rows = stmt.query(params![stale_token])
        .map_err(DatabaseError::FromSQLError)?;

    let mut game_status_map: HashMap<String, GameStatus> = if let Some(row) = rows.next().map_err(DatabaseError::FromSQLError)? {
        let game_status_json: String = row.get(0).map_err(DatabaseError::FromSQLError)?;
        serde_json::from_str(&game_status_json).map_err(|e| DatabaseError::GameStatusParseError(e.to_string()))?
    } else {
        // If no existing entry, create a new HashMap
        HashMap::new()
    };

    // Update the HashMap with the new game status
    game_status_map.insert(key.to_string(), new_game_status.clone());

    // Serialize the updated HashMap back to JSON
    let new_game_status_json = serde_json::to_string(&game_status_map)
        .map_err(|e| DatabaseError::GameStatusParseError(e.to_string()))?;

    // Insert or update the user record in the database
    conn.execute(
        "INSERT INTO users (stale_token, fresh_token, game_status) VALUES (?1, ?2, ?3)
             ON CONFLICT(stale_token) DO UPDATE SET
             fresh_token = excluded.fresh_token,
             game_status = excluded.game_status",
        params![stale_token, new_fresh_token, new_game_status_json],
    )
    .map_err(DatabaseError::FromSQLError)?;

    Ok(new_fresh_token)
}

#[allow(dead_code)]
pub fn get_all_users(conn: &Connection) -> Result<Vec<User>, DatabaseError> {
    let mut stmt = conn.prepare("SELECT stale_token, fresh_token, game_status FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        let game_status: String = row.get(2)?;
        let game_status: HashMap<String, GameStatus> = serde_json::from_str(&game_status)
            .map_err(|_| rusqlite::Error::InvalidQuery)?;

        Ok(User {
            stale_token: row.get(0)?,
            fresh_token: Some(row.get(1)?),
            game_status,
        })
    })?;

    let mut users = Vec::new();
    for user in user_iter {
        users.push(user?);
    }

    Ok(users)
}

#[allow(dead_code)]
pub fn reset_database(conn: &Connection) -> Result<(), DatabaseError> {
    conn.execute("DROP TABLE IF EXISTS users", [])?;
    conn.execute(
        "CREATE TABLE users (
            stale_token TEXT PRIMARY KEY,
            fresh_token TEXT NOT NULL,
            game_status TEXT
        )",
        [],
    )?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::wordguess::WordGuess;

    fn setup_test_db() -> Connection {
        let conn = initialize_connection(true);
        conn.execute(
            "CREATE TABLE users (
                stale_token TEXT PRIMARY KEY,
                fresh_token TEXT NOT NULL,
                game_status TEXT
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_reset_database(){
        let conn = setup_test_db();
        reset_database(&conn).unwrap();
    }

    #[test]
    fn test_create_new_user() {
        let conn = setup_test_db();
        let user_request = UserRequest::NewUser;

        let user = get_user(&conn, user_request).unwrap();

        assert!(user.stale_token.len() > 0);
        assert!(user.fresh_token.is_none());

    }

    #[test]
    fn test_update_user() {
        let conn = setup_test_db();
        let user_request = UserRequest::NewUser;
        // Create new user locally
        let user = get_user(&conn, user_request).unwrap();

        let new_game_status = GameStatus::WordGuess(WordGuess::new());

        let new_token = update_user_game_status(&conn, &user.stale_token, &new_game_status).unwrap();

        // Update the user with the new game status
        let updated_user = get_user(
            &conn,
            UserRequest::Tokens {
                stale_token: user.stale_token.clone(),
                fresh_token: new_token.clone(),
            },
        )
        .unwrap();

        println!("{:?}", updated_user);

        assert_eq!(updated_user.stale_token, user.stale_token);
        assert!(updated_user.fresh_token.is_some());
    }

    #[test]
    fn test_invalid_stale_token() {
        let conn = setup_test_db();

        let result = get_user(
            &conn,
            UserRequest::Tokens {
                stale_token: "invalid_stale".to_string(),
                fresh_token: "invalid_fresh".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fresh_token() {
        let conn = setup_test_db();
        let user_request = UserRequest::NewUser;
        let user = get_user(&conn, user_request).unwrap();

        let result = get_user(
            &conn,
            UserRequest::Tokens {
                stale_token: user.stale_token.clone(),
                fresh_token: "invalid_fresh".to_string(),
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_users() {
        let conn = setup_test_db();

        let users = get_all_users(&conn).unwrap();
        println!("{:?}", users);
    }

    #[test]
    fn test_add_user_and_reset() {
        let conn = setup_test_db();

        // Insert a user
        let user = get_user(&conn, UserRequest::NewUser).unwrap();
        let token = update_user_game_status(
            &conn,
            &user.stale_token,
            &GameStatus::WordGuess(WordGuess::new())
        ).unwrap();

        // Verify user exists
        let user_in_db = get_user(
            &conn,
            UserRequest::Tokens {
                stale_token: user.stale_token.clone(),
                fresh_token: token.clone(),
            },
        ).unwrap();
        
        println!("{:?}", user_in_db);

        // Reset the database
        reset_database(&conn).unwrap();

        // Verify the database is empty
        let users = get_all_users(&conn).unwrap();
        assert_eq!(users.len(), 0);
    }
}
