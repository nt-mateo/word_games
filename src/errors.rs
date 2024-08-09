use thiserror::Error;

/*
    DATABASE ERRORS
*/
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("There's an issue with the request: {0}")]
    FromSQLError(#[from] rusqlite::Error),
    #[error("Unable to parse game status: {0}")]
    GameStatusParseError(String)
}

/*
    GAME ERRORS
*/

#[derive(Error, Debug)]
pub enum GameError {
    #[error("You have reached the maximum number of guesses")]
    MaximumGuesses,
    #[error("You won today's challenge! Try again tomorrow!")]
    GameOver,
    #[error("{0}")]
    InvalidGuess(String),
}