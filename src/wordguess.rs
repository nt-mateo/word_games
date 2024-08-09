use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{errors::GameError, game::Game};

static WORD_OF_THE_DAY: &str = "orate";
static MAXIMUM_GUESSES: usize = 6;
static LETTERS: usize = 5;

/// Represents the condition of a letter in the word
#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
enum Condition {
    NotFound,
    Missplaced,
    Correct,
}

impl Condition {
    #[allow(dead_code)]
    fn to_str(&self) -> &str {
        match self {
            Condition::NotFound => "Not Found",
            Condition::Missplaced => "Missplaced",
            Condition::Correct => "Correct",
        }
    }
}

/// Represents a letter in the guessed word\
/// `value`: The letter\
/// `condition`: The condition of the letter
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Letter {
    value: char,
    condition: Condition,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WordResult {
    letters: Vec<Letter>,
}

impl fmt::Display for WordResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for letter in &self.letters {
            write!(f, "{}", letter.value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WordGuess {
    pub guesses: Vec<WordResult>,
    #[serde(skip)]
    pub answer: String,
    maximum_guesses: usize,
}

impl WordGuess {
    pub fn new() -> Self {
        WordGuess {
            guesses: Vec::new(),
            answer: WORD_OF_THE_DAY.to_string(),
            maximum_guesses: MAXIMUM_GUESSES,
        }
    }

    #[allow(dead_code)]
    pub fn to_vec(&self) -> Vec<String> {
        self.guesses.iter().map(|guess| guess.to_string()).collect()
    }
}

impl Game<&str, String> for WordGuess {
    type State = Self;
    type GameError = GameError;
    type GameResult = WordResult;

    fn guess(&self, guess: &str) -> Result<Self, GameError> {
        self.clean(guess)?;

        let result = self.process(guess.to_string())?;
        Ok(WordGuess {
            guesses: {
                let mut new_guesses = self.guesses.clone();
                new_guesses.push(result);
                new_guesses
            },
            maximum_guesses: self.maximum_guesses,
            answer: self.answer.clone(),
        })
    }

    fn process(&self, guess: String) -> Result<WordResult, GameError> {
        let mut letters = Vec::new();
        // Iterate through the letters
        for (i, c) in guess.chars().enumerate() {
            // Determine if the letter is in the right spot, if it exists, or if it's not found at all
            // Iterate through the answer
            let condition = match self.answer.chars().nth(i) {
                // The answer & guess letter match
                Some(letter) if letter == c => Condition::Correct,
                // The letter was found in the answer, but not the correct position
                Some(_) if self.answer.contains(c) => Condition::Missplaced,
                // It was not found
                _ => Condition::NotFound,
            };

            letters.push(Letter {
                value: c,
                condition,
            });
        }
        Ok(WordResult { letters })
    }

    fn clean(&self, guess: &str) -> Result<String, Self::GameError> {
        // * Maximum guesses
        if self.guesses.len() >= MAXIMUM_GUESSES {
            return Err(GameError::MaximumGuesses);
        }

        // * The guess length is equal to `LETTERS`
        if guess.chars().count() != LETTERS {
            return Err(GameError::InvalidGuess(
                "Guess must be 5 letters".to_string(),
            ));
        }

        // * The guess is a valid word
        if !guess.chars().all(char::is_alphabetic) {
            return Err(GameError::InvalidGuess("Guess must be a word".to_string()));
        }

        // * The guess hasn't been made before
        if self
            .guesses
            .iter()
            .any(|g| g.to_string().to_lowercase() == guess.to_lowercase())
        {
            return Err(GameError::InvalidGuess("Guess already made.".to_string()));
        }

        // * The answer hasn't been guessed
        if self
            .guesses
            .iter()
            .any(|g| g.to_string().to_lowercase() == WORD_OF_THE_DAY)
        {
            return Err(GameError::GameOver);
        }

        Ok(guess.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 6] = ["peets", "steep", "steer", "orate", "radar", "beats"];
    fn setup(answer: Option<&str>) -> WordGuess {
        WordGuess {
            guesses: Vec::new(),
            answer: answer.unwrap_or(WORD_OF_THE_DAY).to_string(),
            maximum_guesses: MAXIMUM_GUESSES,
        }
    }

    #[test]
    fn test_determine_results_perfect() {
        for word in WORDS.iter() {
            let game = setup(Some(word));
            let guess_result = game.guess(word).unwrap();
            let first_guess = guess_result.guesses.first().unwrap();

            // Assert that every letter in the guess is marked as 'Correct'
            for (i, letter) in first_guess.letters.iter().enumerate() {
                assert_eq!(
                    letter.condition,
                    Condition::Correct,
                    "Letter at index {} should be Correct",
                    i
                );
            }
        }
    }

    #[test]
    fn test_determine_results_off_by_1() {
        for word in WORDS.iter() {
            let mut copied: Vec<char> = word.to_string().chars().collect();
            // Corrupt the 3rd letter
            // Ensure that it's different
            match copied[2] {
                'a' => copied[2] = 'b',
                _ => copied[2] = 'a',
            }

            let game = setup(Some(word));
            let guess_result = game.guess(word).unwrap();
            let first_guess = guess_result.guesses.first().unwrap();

            assert_eq!(first_guess.letters[0].condition, Condition::Correct);
            assert_eq!(first_guess.letters[1].condition, Condition::Correct);
            assert_ne!(first_guess.letters[2].condition, Condition::Correct);
            assert_eq!(first_guess.letters[3].condition, Condition::Correct);
            assert_eq!(first_guess.letters[4].condition, Condition::Correct);
        }
    }

    #[test]
    fn test_determine_results_so_wrong_so_right() {
        let game = setup(Some("peets"));
        let guess_result = game.guess("steep").unwrap();
        let first_guess = guess_result.guesses.first().unwrap();
        assert_eq!(first_guess.letters[0].condition, Condition::Missplaced);
        assert_eq!(first_guess.letters[1].condition, Condition::Missplaced);
        assert_eq!(first_guess.letters[2].condition, Condition::Correct);
        assert_eq!(first_guess.letters[3].condition, Condition::Missplaced);
        assert_eq!(first_guess.letters[4].condition, Condition::Missplaced);
    }
}
