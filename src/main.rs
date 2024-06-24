mod game;
mod wordguess;

use errors::{DatabaseError, GameError};
use game::Game;
use models::{GameStatus, User, UserRequest, WordGuessRequest};
mod db;
mod errors;
mod models;
mod token;
use actix_web::{error, post, web, App, HttpResponse, HttpServer, Responder};

async fn get_user(
    req: actix_web::HttpRequest,
    conn: &rusqlite::Connection,
) -> Result<User, DatabaseError> {
    let stale_token = req
        .headers()
        .get("stale-token")
        .and_then(|val| val.to_str().ok().map(String::from));
    let fresh_token = req
        .headers()
        .get("fresh-token")
        .and_then(|val| val.to_str().ok().map(String::from));

    let request = if let (Some(stale_token), Some(fresh_token)) = (stale_token, fresh_token) {
        UserRequest::Tokens {
            stale_token,
            fresh_token,
        }
    } else {
        UserRequest::NewUser
    };

    db::get_user(&conn, request)
}

#[post("/wordguess")]
async fn wordguess_game(
    payload: web::Json<serde_json::Value>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    
    // Deserialize the request
    let data: WordGuessRequest =
        match serde_json::from_value::<WordGuessRequest>(payload.into_inner()) {
            Ok(data) => data,
            Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
        };

    let conn = db::initialize_connection(false);
    let _ = &conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            stale_token TEXT PRIMARY KEY,
            fresh_token TEXT NOT NULL,
            game_status TEXT
        )",
        [],
    );

    let user = match get_user(req, &conn).await {
        Ok(user) => user,
        Err(e) => {
            return match e {
                DatabaseError::FromSQLError(e) => HttpResponse::BadRequest().body(e.to_string()),
                DatabaseError::GameStatusParseError(e) => {
                    HttpResponse::InternalServerError().body(format!(
                        "Your game has been corrupted. Please wait for tomorrow: {}",
                        e
                    ))
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    };

    // Get state or create fresh
    let mut game_status = user.game_status.unwrap_or(GameStatus::new());

    // Initialize WordGuess game
    let mut word_guess = match game_status {
        GameStatus::WordGuess(word_guess) => word_guess,
        _ => panic!("Game status is not WordGuess"),
    };

    // Make a guess
    let result = match word_guess.make_guess(&data.guess) {
        Ok(guess_result) => guess_result,
        Err(e) => {
            return match e {
                GameError::FromWordGuessError(e) => HttpResponse::BadRequest().body(e.to_string()),
                GameError::MaximumGuesses => HttpResponse::BadRequest().body(e.to_string()),
                GameError::GameOver => HttpResponse::Ok().body(e.to_string()),
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    };

    // Update the game status with the new result
    word_guess.guesses.push(result);
    game_status = GameStatus::WordGuess(word_guess);

    // Update the user in the database
    let fresh_token = db::update_user_game_status(&conn, &user.stale_token, &game_status).unwrap();

    // Return the updated user
    HttpResponse::Ok().json(User {
        stale_token: user.stale_token,
        fresh_token: Some(fresh_token),
        game_status: Some(game_status),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    HttpServer::new(|| {
        let json_config = web::JsonConfig::default().limit(75).error_handler(
            |err, _req: &actix_web::HttpRequest| {
                println!("Error: {:?}", err);
                println!("Request: {:?}", _req);
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            },
        );
        App::new().service(wordguess_game).app_data(json_config)
    })
    .bind(("0.0.0.0", 8080))?
    .shutdown_timeout(10)
    .run()
    .await
}
