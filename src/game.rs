/// The main trait for a `Game`
/// ### Type Parameters
/// * `T` - The type of the guess. Usually a primitive type like `u8` or `String`
/// * `U` - The type of the clean. Usually a complex type like a struct or enum
pub trait Game<T, U> {
    type State;
    type GameError;
    type GameResult;

    /// Clean the guess into a complex type
    fn clean(&self, guess: T) -> Result<U, Self::GameError>;

    /// Process the guess and return the result
    fn process(&self, guess: U) -> Result<Self::GameResult, Self::GameError>;

    /// The typical entry point for a game.\
    /// Guess using an expected primitive type\
    /// Returns the new game state
    fn guess(&self, guess: T) -> Result<Self::State, Self::GameError>;
}
