use serde::{Deserialize, Serialize};

use crate::{errors::{GameError, GuessError, WordGuessError}, game::Game, models::WordGuessRequest};

const WORD_OF_THE_DAY: &str = "orate";

/// Represents the condition of a letter in the word
#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Condition {
    NotFound,
    Missplaced,
    Correct,
}

impl Condition {
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
#[derive(Debug, Serialize, Deserialize)]
struct Letter {
    value: char,
    condition: Condition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GuessResult {
    letters: Vec<Letter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WordGuess {
    pub guesses: Vec<GuessResult>
}

impl WordGuess {
    /// Create a new WordGuess game\
    /// ### Parameters
    /// `guesses`: The guesses that have been made. If none are provided, an empty vector is used
    pub fn new() -> Self {
        WordGuess {
            guesses: Vec::new()
        }
    }

    fn determine_results(guess: &str, actual: &str) -> GuessResult {
        let mut letters = Vec::new();
        // Iterate through the letters
        for (i, c) in guess.chars().enumerate() {
            // Determine if the letter is in the right spot, if it exists, or if it's not found at all
            // Iterate through the answer
            let condition = match actual.chars().nth(i) {
                // The answer & guess letter match
                Some(letter) if letter == c => Condition::Correct,
                // The letter was found in the answer, but not the correct position
                Some(_) if actual.contains(c) => Condition::Missplaced,
                // It was not found
                _ => Condition::NotFound,
            };
            letters.push(Letter {
                value: c,
                condition,
            });
        }
        GuessResult { letters }
    }
    
}

impl Game for WordGuess {
    type Guess = String;
    type GameError = GameError;
    type GameResult = GuessResult;

    fn make_guess(&self, guess: &str) -> Result<GuessResult, GameError> {
        // Check if game is already over
        self.is_game_over()?;
        // Confirm validity of the guess
        let guess = WordGuessRequest::try_from(guess.to_string())?;
        // Determine the results of the guess
        Ok(WordGuess::determine_results(&guess.guess, WORD_OF_THE_DAY))
    }
    

    fn get_score(&self) -> u16 {
        self.guesses.len() as u16
    }

    fn is_game_over(&self) -> Result<(), GameError> {

        if self.guesses.len() >= 6 {
            return Err(GameError::MaximumGuesses);
        }
        if self.guesses.last().map_or(false, |result| {
            result.letters.iter().all(|letter| letter.condition == Condition::Correct)
        }) {
            return Err(GameError::GameOver);
        }

        Ok(())
    }
}

impl TryFrom<String> for WordGuessRequest {
    type Error = WordGuessError;

    fn try_from(value: String) -> Result<WordGuessRequest, WordGuessError> {
        // Confirm length of the guess
        if value.len() != 5 {
            return Err(GuessError::InvalidLength.into());
        }
        // Confirm that each character is exclusively a letter
        if !value.chars().all(char::is_alphabetic) {
            return Err(GuessError::InvalidWord.into());
        }

        return Ok(WordGuessRequest {
            guess: value.to_lowercase(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: [&str; 6] = ["peets", "steep", "steer", "orate", "radar", "beats"];

    #[test]
    fn test_determine_results_perfect() {
        for word in WORDS.iter() {
            let result = WordGuess::determine_results(word, word);
            assert_eq!(result.letters[0].condition, Condition::Correct);
            assert_eq!(result.letters[1].condition, Condition::Correct);
            assert_eq!(result.letters[2].condition, Condition::Correct);
            assert_eq!(result.letters[3].condition, Condition::Correct);
            assert_eq!(result.letters[4].condition, Condition::Correct);
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

            let result =
                WordGuess::determine_results(&copied.into_iter().collect::<String>(), word);
            assert_eq!(result.letters[0].condition, Condition::Correct);
            assert_eq!(result.letters[1].condition, Condition::Correct);
            assert_ne!(result.letters[2].condition, Condition::Correct);
            assert_eq!(result.letters[3].condition, Condition::Correct);
            assert_eq!(result.letters[4].condition, Condition::Correct);
        }
    }

    #[test]
    fn test_determine_results_so_wrong_so_right() {
        let result = WordGuess::determine_results("peets", "steep");
        assert_eq!(result.letters[0].condition, Condition::Missplaced);
        assert_eq!(result.letters[1].condition, Condition::Missplaced);
        assert_eq!(result.letters[2].condition, Condition::Correct);
        assert_eq!(result.letters[3].condition, Condition::Missplaced);
        assert_eq!(result.letters[4].condition, Condition::Missplaced);
    }
}
