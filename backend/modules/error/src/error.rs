use actix_web::{Error, HttpRequest, HttpResponse, error::JsonPayloadError};
use argon2::password_hash::Error as Argon2HashError;
use core::fmt;
use sea_orm::DbErr;
use serde_json::json;
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Debug)]
pub enum ApiError {
    InvalidCredentials,
    DatabaseError(DbErr),
    NotFound(String),
    ValidationError(ValidationErrors),
    PasswordHashError(Argon2HashError),
    /// Error parsing PGN format
    PgnParseError(String),
    /// Illegal move detected in PGN
    IllegalMoveError {
        move_number: usize,
        move_text: String,
        reason: String,
    },
}

impl From<DbErr> for ApiError {
    fn from(value: DbErr) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<Argon2HashError> for ApiError {
    fn from(value: Argon2HashError) -> Self {
        Self::PasswordHashError(value)
    }
}

fn parse_validation_error(error_kind: &ValidationErrorsKind, field_name: &str) -> String {
    match error_kind {
        ValidationErrorsKind::Field(field) => {
            if let Some(msg) = &field[0].message {
                format!("{} in list: {}. ", field_name, msg)
            } else {
                format!("Invalid value in list field {}. ", field_name)
            }
        }
        _ => format!("Invalid value in list field {}. ", field_name),
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::InvalidCredentials => write!(f, "Invalid credentials"),
            ApiError::NotFound(v) => write!(f, "{} not found", v),
            ApiError::DatabaseError(err) => write!(f, "Database error {}", err.to_string()),
            ApiError::ValidationError(errs) => {
                let mut s = String::new();
                for (_, error_kind) in errs.errors() {
                    match error_kind {
                        ValidationErrorsKind::Field(field) => {
                            if let Some(message) = &field[0].message {
                                s.push_str(format!("{}. ", message).as_str());
                            } else {
                                s.push_str("Invalid field value. ");
                            }
                        }
                        ValidationErrorsKind::Struct(strct) => {
                            strct.errors().iter().for_each(|(field_name, error_kind)| {
                                s.push_str(&parse_validation_error(error_kind, &field_name))
                            })
                        }
                        ValidationErrorsKind::List(tree) => {
                            tree.iter().for_each(|(_, box_errors)|{
                                box_errors.errors().iter().for_each(|(field_name, error_kind)|{
                                    s.push_str(&parse_validation_error(error_kind, &field_name))
                                })
                            });
                        }
                    }
                }
                write!(f, "{}", s)
            }
            ApiError::PasswordHashError(err) => {
                write!(f, "Unable to hash password: {}", err.to_string())
            }
            ApiError::PgnParseError(msg) => {
                write!(f, "Invalid PGN format: {}", msg)
            }
            ApiError::IllegalMoveError { move_number, move_text, reason } => {
                write!(f, "Illegal move at move {}: '{}' - {}", move_number, move_text, reason)
            }
        }
    }
}

impl ApiError {
    pub fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            ApiError::InvalidCredentials => HttpResponse::BadRequest().json(json!({
                "error": self.to_string(),
                "code": 400
            })),
            ApiError::NotFound(_) => HttpResponse::NotFound().json(json!({
                "error": self.to_string(),
                "code": 404
            })),
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json(json!({
                "error": self.to_string(),
                "code":500
            })),
            ApiError::ValidationError(_) => HttpResponse::BadRequest().json(json!({
                "error": self.to_string(),
                "code":400
            })),
            ApiError::PasswordHashError(_) => HttpResponse::InternalServerError().json(json!({
                "error": self.to_string(),
                "code":500
            })),
            ApiError::PgnParseError(_) => HttpResponse::BadRequest().json(json!({
                "error": self.to_string(),
                "code": 400
            })),
            ApiError::IllegalMoveError { .. } => HttpResponse::UnprocessableEntity().json(json!({
                "error": self.to_string(),
                "code": 422
            })),
        }
    }
}

pub fn custom_json_error(err: JsonPayloadError, _: &HttpRequest) -> Error {
    let error_response = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().json(json!({
            "error":"Invalid Content-Type. Expecting application/json",
            "code": 415
        })),
        // JsonPayloadError::Deserialize(err) => HttpResponse::BadRequest().json(json!({
        //     "error":err.to_string()
        // })),
        _ => HttpResponse::BadRequest().json(json!({
            "error":err.to_string(),
            "code":400
        })),
    };

    actix_web::error::InternalError::from_response(err, error_response).into()
}
