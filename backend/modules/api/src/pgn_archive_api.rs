use actix_web::{web, HttpResponse, post};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::pgn_archive::{PgnArchiveService, PgnArchiveResult};
use dto::auth::ErrorResponse;

#[derive(Debug, Deserialize, ToSchema, Validate)]
pub struct ArchivePgnRequest {
    #[schema(example = "game-123-abc")]
    pub game_id: String,

    #[validate(length(min = 10, message = "PGN content is too short"))]
    #[schema(example = "[Event \"Casual Game\"]\n[Site \"XLMate\"]\n[Date \"2026.02.24\"]\n[Round \"-\"]\n[White \"Player1\"]\n[Black \"Player2\"]\n[Result \"1-0\"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 1-0")]
    pub pgn_content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ArchivePgnResponse {
    pub success: bool,
    pub archive_result: Option<PgnArchiveResult>,
    pub message: String,
}

/// Archive a game PGN to decentralized storage (IPFS/Arweave)
#[utoipa::path(
    post,
    path = "/v1/games/archive-pgn",
    request_body = ArchivePgnRequest,
    responses(
        (status = 200, description = "PGN archived successfully", body = ArchivePgnResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Archive failed", body = ErrorResponse)
    ),
    tag = "Games"
)]
#[post("/archive-pgn")]
pub async fn archive_pgn(
    payload: web::Json<ArchivePgnRequest>,
) -> HttpResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            message: format!("Validation failed: {:?}", errors),
            code: "VALIDATION_ERROR".to_string(),
        });
    }

    // Create archival service
    let archive_service = PgnArchiveService::new();

    // Archive PGN to decentralized storage
    match archive_service.archive_pgn(&payload.game_id, &payload.pgn_content).await {
        Ok(archive_result) => {
            HttpResponse::Ok().json(ArchivePgnResponse {
                success: true,
                archive_result: Some(archive_result),
                message: "PGN archived successfully to decentralized storage".to_string(),
            })
        }
        Err(e) => {
            log::error!("Failed to archive PGN: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                message: format!("Failed to archive PGN: {}", e),
                code: "ARCHIVE_ERROR".to_string(),
            })
        }
    }
}

/// Batch archive multiple PGNs (for completed games)
#[utoipa::path(
    post,
    path = "/v1/games/batch-archive-pgn",
    request_body = Vec<ArchivePgnRequest>,
    responses(
        (status = 200, description = "Batch archive completed", body = Vec<ArchivePgnResponse>),
        (status = 500, description = "Batch archive failed", body = ErrorResponse)
    ),
    tag = "Games"
)]
#[post("/batch-archive-pgn")]
pub async fn batch_archive_pgn(
    payload: web::Json<Vec<ArchivePgnRequest>>,
) -> HttpResponse {
    let archive_service = PgnArchiveService::new();
    let mut results = Vec::new();

    for request in payload.into_inner() {
        match archive_service.archive_pgn(&request.game_id, &request.pgn_content).await {
            Ok(archive_result) => {
                results.push(ArchivePgnResponse {
                    success: true,
                    archive_result: Some(archive_result),
                    message: "PGN archived successfully".to_string(),
                });
            }
            Err(e) => {
                log::warn!("Failed to archive PGN for game {}: {}", request.game_id, e);
                results.push(ArchivePgnResponse {
                    success: false,
                    archive_result: None,
                    message: format!("Failed to archive: {}", e),
                });
            }
        }
    }

    HttpResponse::Ok().json(results)
}
