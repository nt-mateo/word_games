pub trait Game {
    type Guess;
    type GameError;
    type GameResult;

    fn make_guess(&self, guess: &str) -> Result<Self::GameResult, Self::GameError>;
    fn get_score(&self) -> u16;
    fn is_game_over(&self) -> bool;
}
