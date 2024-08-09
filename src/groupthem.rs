use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use crate::{
    errors::GameError, game::Game, models::{mix_colors, GroupResult, Word}
};

static MAXIMUM_BAD_GUESSES: u8 = 4;
static GROUPS: u8 = 4;
static ITEMS_PER_GROUP: usize = 4;

/// Represents the game state for the user
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GroupThem {
    pub guesses: Vec<GroupResult>,
    pub available_words: Vec<Word>,
}

impl GroupThem {
    pub fn new(words: &[Word]) -> Self {
        GroupThem {
            guesses: Vec::new(),
            available_words: words.to_vec(),
        }
    }

    fn count_bad_guesses(&self) -> u8 {
        self.guesses
            .iter()
            .filter(|g| !g.is_group())
            .count() as u8
    }

    fn count_good_guesses(&self) -> u8 {
        self.guesses
            .iter()
            .filter(|g| g.is_group())
            .count() as u8
    }

    fn good_guesses(&self) -> Vec<Vec<String>> {
        self.guesses
            .iter()
            .filter(|g| g.is_group())
            .map(|g| g.words.iter().map(|w| w.text.clone()).collect())
            .collect()
    }
}

impl Game<Vec<String>, Vec<Word>> for GroupThem {
    type State = Self;
    type GameError = GameError;
    type GameResult = Vec<Word>;

    fn process(&self, guess: Vec<Word>) -> Result<Self::GameResult, Self::GameError> {
        if guess.iter().all(|w| w.group == guess[0].group) {
            // Was a correct guess
            // Remove the used words from the available words
            let available_words: Vec<Word> = self
                .available_words
                .iter()
                .filter(|w| !guess.iter().any(|g| g.text == w.text))
                .cloned()
                .collect();
            Ok(available_words)
        } else {
            Ok(
                self.available_words.clone()
            )
        }
    }

    fn clean(&self, guess: Vec<String>) -> Result<Vec<Word>, Self::GameError> {
        let guess_set = guess
            .iter()
            .map(|g| g.to_string())
            .collect::<HashSet<String>>();

        // * There wasn't 4 guesses made
        if guess_set.len() != ITEMS_PER_GROUP {
            Err(GameError::InvalidGuess(
                "You have to guess 4 words".to_string(),
            ))?
        }

        // * The game has already been won
        if self.count_good_guesses() == GROUPS {
            Err(GameError::GameOver)?
        }

        // * Exceeded the maximum number of bad guesses
        if self.count_bad_guesses() >= MAXIMUM_BAD_GUESSES {
            Err(GameError::MaximumGuesses)?
        }

        // Convert each `guess` to `Word`
        let words = guess_set
            .iter()
            .map(|g| Word::try_from(
                g,
                &self.available_words
            ))
            .collect::<Result<Vec<Word>, GameError>>()?;


        // * A word hasn't been already correctly used
        for group in &self.good_guesses() {
            for guess in group {
                if guess_set.contains(guess) {
                    return Err(GameError::InvalidGuess(format!(
                        "Word has already been correctly used: {}",
                        guess
                    )));
                }
            }
        }

        // * The guess was made before
        if self
            .guesses
            .iter()
            .any(|g| g.words.iter().all(|w| words.iter().any(|word| word.text == w.text)))
        {
            Err(GameError::InvalidGuess("Guess already made.".to_string()))?
        }
        
        Ok(words)
    }

    fn guess(&self, guess: Vec<String>) -> Result<Self::State, Self::GameError> {
        let words = self.clean(guess)?;
        let available_words = self.process(words.clone())?;

        Ok(GroupThem {
            guesses: {
                let mut new_guesses = self.guesses.clone();
                new_guesses.push(GroupResult {
                    color: mix_colors(&words),
                    words
                });
                new_guesses
            },
            available_words,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Group, Ranking};
    use super::*;

    fn setup() -> (Vec<Group>, Vec<Word>) {
        let groups = [
            Group {
                name: "common desserts".to_string(),
                ranking: Ranking::Easy,
            },
            Group {
                name: "items found in a stationery store".to_string(),
                ranking: Ranking::Medium,
            },
            Group {
                name: "famous detectives in literature".to_string(),
                ranking: Ranking::Hard,
            },
            Group {
                name: "mythical creatures with human traits".to_string(),
                ranking: Ranking::VeryDifficult,
            },
        ];

        let all_words = [
            Word {
                text: "cake".to_string(),
                group: groups[0].clone(),
            },
            Word {
                text: "pie".to_string(),
                group: groups[0].clone(),
            },
            Word {
                text: "pudding".to_string(),
                group: groups[0].clone(),
            },
            Word {
                text: "cookie".to_string(),
                group: groups[0].clone(),
            },
            Word {
                text: "pen".to_string(),
                group: groups[1].clone(),
            },
            Word {
                text: "notebook".to_string(),
                group: groups[1].clone(),
            },
            Word {
                text: "stapler".to_string(),
                group: groups[1].clone(),
            },
            Word {
                text: "envelope".to_string(),
                group: groups[1].clone(),
            },
            Word {
                text: "holmes".to_string(),
                group: groups[2].clone(),
            },
            Word {
                text: "poirot".to_string(),
                group: groups[2].clone(),
            },
            Word {
                text: "marple".to_string(),
                group: groups[2].clone(),
            },
            Word {
                text: "spade".to_string(),
                group: groups[2].clone(),
            },
            Word {
                text: "centaur".to_string(),
                group: groups[3].clone(),
            },
            Word {
                text: "mermaid".to_string(),
                group: groups[3].clone(),
            },
            Word {
                text: "minotaur".to_string(),
                group: groups[3].clone(),
            },
            Word {
                text: "sphinx".to_string(),
                group: groups[3].clone(),
            },
        ];

        (groups.to_vec(), all_words.to_vec())
    }

    #[test]
    fn test_try_bad_word() {
        let (_, all_words) = setup();
        let result = Word::try_from("sdfasfasdfasdfsadfasd", &all_words);
        assert!(result.is_err());

        let bad_words = vec![
            "sdfasfasdfasdfsadfasd".to_string(),
            "sdfasfasdfasdfsadfasd".to_string(),
        ];
        let game = GroupThem::new(&all_words);
        let result = game.guess(bad_words);

        assert!(result.is_err());
    }

    #[test]
    fn test_guess_correct_group() {
        let (_, all_words) = setup();
        let game = GroupThem::new(&all_words);
        let result = game.guess(vec![
            "cake".to_string(),
            "pie".to_string(),
            "pudding".to_string(),
            "cookie".to_string(),
        ]);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.guesses.len(), 1);

        let good_guesses = result.good_guesses();
        assert_eq!(good_guesses.len(), 1);

    }

    #[test]
    fn test_guess_incorrect_group() {
        let (_, all_words) = setup();
        let game = GroupThem::new(&all_words);
        let result = game.guess(vec![
            "cake".to_string(),
            "pie".to_string(),
            "pudding".to_string(),
            "pen".to_string(),
        ]);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.guesses.len(), 1);
        
        let bad_guesses = result.count_bad_guesses();
        assert_eq!(1, bad_guesses);
    }

    #[test]
    fn test_guess_one_correct_one_incorrect() {
        let guesses = [
            vec![
                "cake".to_string(),
                "pie".to_string(),
                "pudding".to_string(),
                "cookie".to_string(),
            ],
            vec![
                "sphinx".to_string(),
                "minotaur".to_string(),
                "mermaid".to_string(),
                "envelope".to_string(),
            ],
        ];

        let (_, all_words) = setup();
        let mut game = GroupThem::new(&all_words);
        for guess in guesses.iter() {
            game.guesses.push(game.guess(guess.to_owned()).unwrap().guesses.last().unwrap().clone());
        }

        assert_eq!(game.guesses.len(), 2);

        let good_guesses = game.count_good_guesses();
        assert_eq!(1, good_guesses);

        let bad_guesses = game.count_bad_guesses();
        assert_eq!(bad_guesses, 1);

    }

    #[test]
    fn test_repeat_guess(){
        let (_, all_words) = setup();
        let mut game = GroupThem::new(&all_words);

        let result = game.guess(vec![
            "cake".to_string(),
            "pie".to_string(),
            "pudding".to_string(),
            "sphinx".to_string(),
        ]).unwrap().guesses.last().unwrap().clone();

        game.guesses.push(result);

        assert_eq!(game.guesses.len(), 1);

        // Confirm is bad
        let bad_guesses = game.count_bad_guesses();
        assert_eq!(bad_guesses, 1);

        // Make a guess with `cookie` again
        let result = game.guess(vec![
            "cake".to_string(),
            "pie".to_string(),
            "pudding".to_string(),
            "sphinx".to_string(),
        ]);

        println!("{:?}", result);

        assert!(result.is_err());
    }

    #[test]
    fn test_ran_out_of_guesses() {
        let (_, all_words) = setup();
        let mut game = GroupThem::new(&all_words);

        for i in 0..MAXIMUM_BAD_GUESSES {
            let result = game.guess(vec![
                all_words[0].text.clone(),
                all_words[1].text.clone(),
                all_words[2].text.clone(),
                all_words[all_words.len() - 1 - i as usize].text.clone(),
            ]).unwrap().guesses.last().unwrap().clone();

            game.guesses.push(result);
        }

        assert_eq!(game.guesses.len(), MAXIMUM_BAD_GUESSES as usize);

        // Make a bad guess
        let result = game.guess(vec![
            "sphinx".to_string(),
            "minotaur".to_string(),
            "mermaid".to_string(),
            "envelope".to_string(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_guess_already_guessed_word() {
        let (_, all_words) = setup();
        let game = GroupThem::new(&all_words);
        let result = game.guess(vec![
            "cake".to_string(),
            "pie".to_string(),
            "pudding".to_string(),
            "cookie".to_string(),
        ]);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.guesses.len(), 1);

        // Confirm is good
        let good_guesses = result.count_good_guesses();
        assert_eq!(good_guesses, 1);

        // Make a guess with `cookie` again
        let result = result.guess(vec![
            "cookie".to_string(),
            "minotaur".to_string(),
            "mermaid".to_string(),
            "sphinx".to_string(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn test_game_over() {
        let (_, all_words) = setup();
        let mut game = GroupThem::new(&all_words);

        for i in 0..GROUPS {
            let result = game.guess(vec![
                all_words[i as usize * ITEMS_PER_GROUP].text.clone(),
                all_words[i as usize * ITEMS_PER_GROUP + 1].text.clone(),
                all_words[i as usize * ITEMS_PER_GROUP + 2].text.clone(),
                all_words[i as usize * ITEMS_PER_GROUP + 3].text.clone(),
            ]).unwrap().guesses.last().unwrap().clone();

            game.guesses.push(result);
        }

        assert_eq!(game.guesses.len(), GROUPS as usize);

        // Make a bad guess
        let result = game.guess(vec![
            "sphinx".to_string(),
            "minotaur".to_string(),
            "mermaid".to_string(),
            "envelope".to_string(),
        ]);

        assert!(result.is_err());
    }
}
