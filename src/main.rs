mod game;
mod groupthem;
mod wordguess;
use errors::{DatabaseError, GameError};
use game::Game;
use groupthem::{get_data, GroupThem};
use models::{GameStatus, GroupThemRequest, User, UserRequest, WordGuessRequest};
mod db;
mod errors;
mod models;
mod token;
use actix_web::{cookie::Cookie, web, App, HttpResponse, HttpServer, Responder};
use serde_json::json;
use wordguess::WordGuess;

async fn get_user(
    req: actix_web::HttpRequest,
    conn: &rusqlite::Connection,
) -> Result<User, DatabaseError> {
    let stale_token = req
        .cookie("stale_token")
        .map(|cookie| cookie.value().to_string());
    let fresh_token = req
        .cookie("fresh_token")
        .map(|cookie| cookie.value().to_string());

    let request = if let (Some(stale_token), Some(fresh_token)) = (stale_token, fresh_token) {
        UserRequest::Tokens {
            stale_token,
            fresh_token,
        }
    } else {
        UserRequest::NewUser
    };

    db::get_user(conn, request)
}

async fn groupthem_get_state(
    req: actix_web::HttpRequest
) -> impl Responder {
    
    let conn = db::initialize_connection(false);

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
            }
        }
    };

    let game = get_data(&conn, 1).await.unwrap();

    let state = match user.game_status.get("group_them") {
        Some(GameStatus::GroupThem(group_item)) => group_item,
        _ => &GroupThem::new(&game.1),
    };

    HttpResponse::Ok().json(state)
}

async fn groupthem_game(
    payload: web::Json<serde_json::Value>,
    req: actix_web::HttpRequest
) -> impl Responder {
    // Deserialize the request
    let request = match serde_json::from_value::<GroupThemRequest>(payload.into_inner()) {
        Ok(data) => data,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let conn = db::initialize_connection(false);

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
            }
        }
    };

    let game = get_data(&conn, 1).await.unwrap();


    let state = match user.game_status.get("group_them") {
        Some(GameStatus::GroupThem(group_item)) => group_item,
        _ => &GroupThem::new(&game.1),
    };


    // Make a guess
    let result = match state.guess(request.guess) {
        Ok(guess_result) => guess_result,
        Err(e) => {
            return match e {
                GameError::MaximumGuesses => HttpResponse::BadRequest().body(e.to_string()),
                GameError::GameOver => HttpResponse::Ok().body(e.to_string()),
                GameError::InvalidGuess(e) => HttpResponse::BadRequest().body(e.to_string()),
                GameError::NetworkError(e) => HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    };

    // Update the game status with the new result
    let game_status = GameStatus::GroupThem(result);

    // Update the user in the database
    let fresh_token = db::update_user_game_status(
        &conn,
        &user.stale_token,
        &game_status
    ).unwrap();

    let mut response = HttpResponse::Ok().json(game_status);

    let _ = response.add_cookie(
        &Cookie::build("fresh_token", fresh_token).path("/")
        .http_only(true)
        .finish()
    );
    let _ = response.add_cookie(
        &Cookie::build("stale_token", user.stale_token).path("/")
        .http_only(true)
        .finish()
    );

    response
}

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
            }
        }
    };

    let state = match user.game_status.get("word_guess") {
        Some(GameStatus::WordGuess(word_guess)) => word_guess,
        _ => &WordGuess::new(),
    };

    // Make a guess
    let result = match state.guess(&data.guess) {
        Ok(guess_result) => guess_result,
        Err(e) => {
            return match e {
                GameError::MaximumGuesses => HttpResponse::BadRequest().body(e.to_string()),
                GameError::GameOver => HttpResponse::Ok().body(e.to_string()),
                GameError::InvalidGuess(e) => HttpResponse::BadRequest().body(e.to_string()),
                GameError::NetworkError(e) => HttpResponse::InternalServerError().body(e.to_string())
            }
        }
    };

    let game_status = GameStatus::WordGuess(result);
    
    // Update the user in the database
    let fresh_token = db::update_user_game_status(
        &conn,
        &user.stale_token,
        &game_status
    ).unwrap();

    let mut response = HttpResponse::Ok().json(json!({
        "game_status": game_status,
    }));

    let _ = response.add_cookie(
        &Cookie::build("fresh_token", fresh_token).path("/")
        .http_only(true)
        .finish()
    );
    let _ = response.add_cookie(
        &Cookie::build("stale_token", user.stale_token).path("/")
        .http_only(true)
        .finish()
    );

    response
}

async fn wordguess_get_state(
    req: actix_web::HttpRequest
) -> impl Responder {
    
    let conn = db::initialize_connection(false);

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
            }
        }
    };

    let state = match user.game_status.get("word_guess") {
        Some(GameStatus::WordGuess(word_guess)) => word_guess,
        _ => &WordGuess::new(),
    };

    HttpResponse::Ok().json(state)
}

async fn get_schema(path: web::Path<String>) -> impl Responder {
    match path.to_lowercase().as_str() {
        "wordguess" => HttpResponse::Ok().body(WordGuessRequest::schema()),
        "groupthem" => HttpResponse::Ok().body(GroupThemRequest::schema()),
        _ => HttpResponse::NotFound().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    HttpServer::new(|| {
        let json_config = web::JsonConfig::default().limit(200);
        App::new()
            .service(
                web::resource("/wordguess")
                    .app_data(json_config.clone())
                    // POST /wordguess
                    // Make a guess in the word guess game
                    .route(web::post().to(wordguess_game))
                    // GET /wordguess
                    // Get the current state of the word guess game
                    .route(web::get().to(wordguess_get_state))
            )
            .service(
                web::resource("/{segment}/schema")
                    // GET /{segment}/schema
                    // Get the schema used for POST requests
                    // Ex. GET /wordguess/schema
                    .route(web::get().to(get_schema)),
            )
            .service(
                web::resource("/groupthem")
                    .app_data(json_config.clone())
                    .route(web::post().to(groupthem_game))
                    .route(web::get().to(groupthem_get_state))
            )
            .app_data(json_config)
    })
    .bind(("127.0.0.1", 8080))?
    .shutdown_timeout(10)
    .run()
    .await
}
