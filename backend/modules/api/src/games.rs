use actix_web::{
    HttpResponse, HttpRequest, delete, get, post, put,
    web::{self, Json, Path, Query},
};
use dto::{
    games::{
        CreateGameRequest, GameDisplayDTO, MakeMoveRequest, JoinGameRequest,
        GameStatus, ListGamesQuery, ImportGameRequest, ImportGameResponse,
    },
    responses::{InvalidCredentialsResponse, NotFoundResponse},
};
use error::error::ApiError;
use serde_json::json;
use validator::Validate;
use uuid::Uuid;
use sea_orm::DatabaseConnection;
use service::games::GameService;

// ---------------------------------------------------------------------------
// Helper: extract authenticated player UUID inserted by the JWT middleware.
// ---------------------------------------------------------------------------
fn authenticated_player(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(json!({
                "message": "Authentication required"
            }))
        })
}

// ---------------------------------------------------------------------------
// POST /v1/games
// ---------------------------------------------------------------------------
#[utoipa::path(
    post,
    path = "/v1/games",
    request_body = CreateGameRequest,
    responses(
        (status = 201, description = "Game created successfully",  body = GameDisplayDTO),
        (status = 400, description = "Invalid request parameters", body = InvalidCredentialsResponse),
        (status = 401, description = "Unauthorized",               body = InvalidCredentialsResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[post("")]
pub async fn create_game(
    req: HttpRequest,
    payload: Json<CreateGameRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    if let Err(errors) = payload.0.validate() {
        return ApiError::ValidationError(errors).error_response();
    }

    let creator_id = match authenticated_player(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    match GameService::create_game(db.get_ref(), creator_id, payload.0).await {
        Ok(game_dto) => HttpResponse::Created().json(json!({
            "message": "Game created successfully",
            "data": { "game": game_dto }
        })),
        Err(e) => {
            eprintln!("create_game error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to create game"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/games/{id}
// ---------------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/v1/games/{id}",
    params(
        ("id" = String, Path, description = "Game ID in UUID format", format = "uuid")
    ),
    responses(
        (status = 200, description = "Game found",     body = GameDisplayDTO),
        (status = 404, description = "Game not found", body = NotFoundResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[get("/{id}")]
pub async fn get_game(
    id: Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let game_id = id.into_inner();

    match GameService::get_game(db.get_ref(), game_id).await {
        Ok(Some(game_dto)) => HttpResponse::Ok().json(json!({
            "message": "Game found",
            "data": { "game": game_dto }
        })),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "message": "Game not found"
        })),
        Err(e) => {
            eprintln!("get_game error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to fetch game"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// PUT /v1/games/{id}/move
// ---------------------------------------------------------------------------
#[utoipa::path(
    put,
    path = "/v1/games/{id}/move",
    params(
        ("id" = String, Path, description = "Game ID in UUID format", format = "uuid")
    ),
    request_body = MakeMoveRequest,
    responses(
        (status = 200, description = "Move made successfully", body = GameDisplayDTO),
        (status = 400, description = "Invalid move",           body = InvalidCredentialsResponse),
        (status = 404, description = "Game not found",         body = NotFoundResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[put("/{id}/move")]
pub async fn make_move(
    req: HttpRequest,
    id: Path<Uuid>,
    payload: Json<MakeMoveRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    if let Err(errors) = payload.0.validate() {
        return ApiError::ValidationError(errors).error_response();
    }

    let player_id = match authenticated_player(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let game_id = id.into_inner();

    match GameService::make_move(db.get_ref(), game_id, player_id, payload.0).await {
        Ok(game_dto) => HttpResponse::Ok().json(json!({
            "message": "Move made successfully",
            "data": { "game": game_dto }
        })),
        Err(ApiError::NotFound(_)) => HttpResponse::NotFound().json(json!({
            "message": "Game not found"
        })),
        Err(ApiError::BadRequest(msg)) => HttpResponse::BadRequest().json(json!({
            "message": msg
        })),
        Err(ApiError::Forbidden(_)) => HttpResponse::Forbidden().json(json!({
            "message": "It is not your turn"
        })),
        Err(e) => {
            eprintln!("make_move error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to apply move"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// GET /v1/games
// ---------------------------------------------------------------------------
#[utoipa::path(
    get,
    path = "/v1/games",
    params(
        ("status"    = Option<String>, Query, description = "Filter by status (waiting, in_progress, completed, aborted)"),
        ("player_id" = Option<String>, Query, description = "Filter by player UUID", format = "uuid"),
        ("page"      = Option<i32>,    Query, description = "Page number"),
        ("limit"     = Option<i32>,    Query, description = "Items per page")
    ),
    responses(
        (status = 200, description = "List of games", body = Vec<GameDisplayDTO>)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[get("")]
pub async fn list_games(
    query: Query<ListGamesQuery>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let status_enum: Option<GameStatus> = query.status.as_deref().and_then(|s| match s {
        "waiting"     => Some(GameStatus::Waiting),
        "in_progress" => Some(GameStatus::InProgress),
        "completed"   => Some(GameStatus::Completed),
        "aborted"     => Some(GameStatus::Aborted),
        _             => None,
    });

    let limit  = query.limit.unwrap_or(10);
    let cursor = query.cursor.clone();

    match GameService::list_games(
        db.get_ref(),
        cursor,
        limit,
        query.player_id,
        status_enum,
    )
    .await
    {
        Ok((games, next_cursor)) => {
            let game_dtos: Vec<serde_json::Value> = games
                .into_iter()
                .map(|g| {
                    let status = match &g.result {
                        Some(_)                      => "completed",
                        None if g.started_at.is_some() => "in_progress",
                        None                         => "waiting",
                    };
                    json!({
                        "id":              g.id,
                        "white_player_id": g.white_player,
                        "black_player_id": g.black_player,
                        "status":          status,
                        "result":          g.result,
                        "current_fen":     g.fen,
                        "created_at":      g.created_at,
                        "started_at":      g.started_at,
                    })
                })
                .collect();

            HttpResponse::Ok().json(json!({
                "message": "Games found",
                "data": {
                    "games":       game_dtos,
                    "next_cursor": next_cursor,
                    "limit":       limit,
                }
            }))
        }
        Err(e) => {
            eprintln!("list_games error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to list games"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /v1/games/{id}/join
// ---------------------------------------------------------------------------
#[utoipa::path(
    post,
    path = "/v1/games/{id}/join",
    params(
        ("id" = String, Path, description = "Game ID in UUID format", format = "uuid")
    ),
    request_body = JoinGameRequest,
    responses(
        (status = 200, description = "Joined game successfully", body = GameDisplayDTO),
        (status = 400, description = "Cannot join game",         body = InvalidCredentialsResponse),
        (status = 404, description = "Game not found",           body = NotFoundResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[post("/{id}/join")]
pub async fn join_game(
    req: HttpRequest,
    id: Path<Uuid>,
    payload: Json<JoinGameRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    if let Err(errors) = payload.0.validate() {
        return ApiError::ValidationError(errors).error_response();
    }

    // Prefer JWT-extracted id; fall back to body field so the DTO stays intact.
    let player_id = authenticated_player(&req).unwrap_or(payload.0.player_id);
    let game_id   = id.into_inner();

    match GameService::join_game(db.get_ref(), game_id, player_id).await {
        Ok(game_dto) => HttpResponse::Ok().json(json!({
            "message": "Joined game successfully",
            "data": { "game": game_dto }
        })),
        Err(ApiError::NotFound(_)) => HttpResponse::NotFound().json(json!({
            "message": "Game not found"
        })),
        Err(ApiError::BadRequest(msg)) => HttpResponse::BadRequest().json(json!({
            "message": msg
        })),
        Err(e) => {
            eprintln!("join_game error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to join game"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// DELETE /v1/games/{id}
// ---------------------------------------------------------------------------
#[utoipa::path(
    delete,
    path = "/v1/games/{id}",
    params(
        ("id" = String, Path, description = "Game ID in UUID format", format = "uuid")
    ),
    responses(
        (status = 200, description = "Game abandoned successfully"),
        (status = 404, description = "Game not found",              body = NotFoundResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[delete("/{id}")]
pub async fn abandon_game(
    req: HttpRequest,
    id: Path<Uuid>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    let player_id = match authenticated_player(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let game_id = id.into_inner();

    match GameService::abandon_game(db.get_ref(), game_id, player_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Game abandoned successfully",
            "data": {}
        })),
        Err(ApiError::NotFound(_)) => HttpResponse::NotFound().json(json!({
            "message": "Game not found"
        })),
        Err(ApiError::Forbidden(_)) => HttpResponse::Forbidden().json(json!({
            "message": "You are not a participant in this game"
        })),
        Err(e) => {
            eprintln!("abandon_game error: {e}");
            HttpResponse::InternalServerError().json(json!({
                "message": "Failed to abandon game"
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /v1/games/import
// ---------------------------------------------------------------------------
#[utoipa::path(
    post,
    path = "/v1/games/import",
    request_body = ImportGameRequest,
    responses(
        (status = 201, description = "Game imported successfully", body = ImportGameResponse),
        (status = 400, description = "Invalid PGN format",         body = InvalidCredentialsResponse),
        (status = 422, description = "Illegal moves in PGN",       body = InvalidCredentialsResponse)
    ),
    security(("jwt_auth" = [])),
    tag = "Games"
)]
#[post("/import")]
pub async fn import_game(
    req: HttpRequest,
    payload: Json<ImportGameRequest>,
    db: web::Data<DatabaseConnection>,
) -> HttpResponse {
    if let Err(errors) = payload.0.validate() {
        return ApiError::ValidationError(errors).error_response();
    }

    let importer_id = match authenticated_player(&req) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    // Parse PGN.
    let parsed = match chess::parse_pgn(&payload.pgn) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest().json(ImportGameResponse {
                success:      false,
                game_id:      None,
                white_player: String::new(),
                black_player: String::new(),
                result:       String::new(),
                move_count:   0,
                final_fen:    None,
                error:        Some(e.to_string()),
            });
        }
    };

    // Validate move legality.
    let validated = match chess::validate_game(&parsed) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::UnprocessableEntity().json(ImportGameResponse {
                success:      false,
                game_id:      None,
                white_player: parsed.headers.white.clone(),
                black_player: parsed.headers.black.clone(),
                result:       String::new(),
                move_count:   0,
                final_fen:    None,
                error:        Some(e.to_string()),
            });
        }
    };

    let result_str = validated.headers.result.to_pgn_string().to_string();

    // Persist in DB with is_imported = true.
    match GameService::import_game(db.get_ref(), importer_id, &validated).await {
        Ok(game_id) => HttpResponse::Created().json(ImportGameResponse {
            success:      true,
            game_id:      Some(game_id),
            white_player: validated.headers.white,
            black_player: validated.headers.black,
            result:       result_str,
            move_count:   validated.ply_count,
            final_fen:    Some(validated.final_fen),
            error:        None,
        }),
        Err(e) => {
            eprintln!("import_game DB error: {e}");
            HttpResponse::InternalServerError().json(ImportGameResponse {
                success:      false,
                game_id:      None,
                white_player: validated.headers.white,
                black_player: validated.headers.black,
                result:       result_str,
                move_count:   validated.ply_count,
                final_fen:    Some(validated.final_fen),
                error:        Some("Failed to persist imported game".to_string()),
            })
        }
    }
}