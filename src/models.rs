use core::fmt;
use std::{collections::HashMap, hash::Hash};

use palette::{IntoColor, Lch, Mix, Srgb};
use rand::Rng;
use serde::{Deserialize, Serialize};
use schemars::{schema_for, JsonSchema};
use crate::{errors::GameError, groupthem::GroupThem, wordguess::WordGuess};

/*
    HTTP Request Models
*/
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GroupThemRequest {
    pub guess: Vec<String>,
}

impl GroupThemRequest {
    pub fn schema() -> String {
        serde_json::to_string_pretty(&schema_for!(GroupThemRequest)).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WordGuessRequest {
    pub guess: String,
}

impl WordGuessRequest {
    pub fn schema() -> String {
        serde_json::to_string_pretty(&schema_for!(WordGuessRequest)).unwrap()
    }
}

/*
    END HTTP Request Models
*/

#[derive(Debug, Serialize, Deserialize)]
pub enum GuessInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub stale_token: String,
    pub fresh_token: Option<String>,
    pub game_status: HashMap<String, GameStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub stale: String,
    pub fresh: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserRequest {
    Tokens {
        stale_token: String,
        fresh_token: String,
    },
    NewUser,
}

/*
 * GroupGuess Models
 *
 * A GroupGuess is a result of a guess in the GroupThem game
 * It can either be a `Good` or `Bad` guess
*/

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Ranking {
    Easy,
    Medium,
    Hard,
    VeryDifficult,
}

impl Ranking {
    // Convert to palette Srgb
    fn to_palette(&self) -> Lch {
        match self {
            Ranking::Easy => Srgb::new(0.0, 0.8, 0.0).into_color(),
            Ranking::Medium => Srgb::new(0.0, 0.0, 0.8).into_color(),
            Ranking::Hard => Srgb::new(0.8, 0.0, 0.0).into_color(),
            Ranking::VeryDifficult => Srgb::new(0.8, 0.8, 0.8).into_color(),
        }
    }

    #[allow(dead_code)]
    fn to_str(&self) -> &str {
        match self {
            Ranking::Easy => "Easy",
            Ranking::Medium => "Medium",
            Ranking::Hard => "Hard",
            Ranking::VeryDifficult => "Very Difficult",
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Ranking::Easy,
            1 => Ranking::Medium,
            2 => Ranking::Hard,
            3 => Ranking::VeryDifficult,
            _ => Ranking::Easy,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct ApproxColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ApproxColor {
    #[allow(dead_code)]
    fn new() -> Self {
        ApproxColor {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl From<Lch> for ApproxColor {
    fn from(color: Lch) -> Self {
        let srgb: Srgb = color.into_color();
        ApproxColor {
            red: (srgb.red * 255.0).round() as u8,
            green: (srgb.green * 255.0).round() as u8,
            blue: (srgb.blue * 255.0).round() as u8,
        }
    }
}

/// Mix the colors of the vector
/// ### Parameters
/// `words`: The words to blend
/// ### Returns
/// A tuple containing the RGB values of the mixed color
pub fn mix_colors(words: &[Word]) -> ApproxColor {
    let mut colors = words
        .iter()
        .map(|word| word.to_palette())
        .collect::<Vec<Lch>>();
    let mut color = colors.pop().unwrap();
    for c in colors {
        let mut rng = rand::thread_rng();
        let factor = rng.gen_range(0.2..0.5);
        color = color.mix(c, factor);
    }
    color.into()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Hash, Eq, Clone)]
pub struct Group {
    pub name: String,
    pub ranking: Ranking,
}

impl Group {
    fn to_palette(&self) -> Lch {
        self.ranking.to_palette()
    }
}


#[derive(Debug, PartialEq, Hash, Eq, Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub group: Group,
}

impl Word {
    pub fn to_palette(&self) -> Lch {
        self.group.to_palette()
    }
    pub fn try_from(text: &str, all: &[Word]) -> Result<Self, GameError> {
        if let Some(word) = all.iter().find(|word| word.text == text) {
            Ok(Word {
                text: word.text.clone(),
                group: word.group.clone(),
            })
        } else {
            Err(GameError::InvalidGuess(format!(
                "`{}` is not a valid word",
                text
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, Clone, Eq)]
pub struct GroupResult {
    pub words: Vec<Word>,
    pub color: ApproxColor,
}

impl GroupResult {
    pub fn is_group(&self) -> bool {
        self.words
            .iter()
            .all(|word| word.group == self.words[0].group)
    }
}

#[derive(Debug, Hash, PartialEq, Clone, Eq)]
pub struct GroupData {
    pub words: Vec<Word>,
}

/*
    End of GroupGuess Models
*/

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum GameStatus {
    #[serde(rename = "word_guess")]
    WordGuess(WordGuess),
    #[serde(rename = "group_them")]
    GroupThem(GroupThem),
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameStatus::WordGuess(_) => write!(f, "word_guess"),
            GameStatus::GroupThem(_) => write!(f, "group_them"),
        }
    }
}