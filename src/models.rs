use serde::{Deserialize, Serialize};

use crate::wordguess::WordGuess;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub stale_token: String,
    pub fresh_token: Option<String>,
    pub game_status: Option<GameStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub stale: String,
    pub fresh: String,
}
pub enum UserRequest {
    Tokens {
        stale_token: String,
        fresh_token: String,
    },
    NewUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordGuessRequest {
    pub guess: String
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameStatus {
    #[serde(rename = "word_guess")]
    WordGuess(WordGuess),
}

impl GameStatus {
    pub fn new() -> Self {
        GameStatus::WordGuess(WordGuess::new())
    }
}