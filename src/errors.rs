use thiserror::Error;
use rusqlite;

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
    #[error("{0}")]
    FromWordGuessError(#[from] WordGuessError),
    #[error("You have reached the maximum number of guesses")]
    MaximumGuesses
}


/*
    WORDGUESS ERRORS
*/

#[derive(Error, Debug)]
pub enum WordGuessError {
    #[error("There's an issue with your guess: {0}")]
    FromGuessError(#[from] GuessError),
}

#[derive(Error, Debug)]
pub enum GuessError {
    #[error("Must be 5 characters long")]
    InvalidLength,
    #[error("Must be a valid word")]
    InvalidWord,
}

