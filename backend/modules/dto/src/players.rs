use entity::player::Model;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct NewPlayer {
    #[validate(length(
        min = 4,
        max = 20,
        message = "Username must be between 4 and 20 characters"
    ))]
    pub username: String,

    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 64,
        message = "Password must be between 8 and 64 characters"
    ))]
    pub password: String,

    #[validate(length(max = 100, message = "Real name must be less than 100 characters"))]
    pub real_name: String,
}

pub enum InvalidPlayer {
    Email,
    Password,
    Username,
}

impl NewPlayer {
    pub fn test_player() -> Self {
        let rnd: i32 = rand::random();
        Self {
            username: format!("Player {}", rnd),
            email: format!("player{}@gmail.com", rnd),
            password: format!("PasswordIsVeryStrong"),
            real_name: format!("A new player"),
        }
    }

    pub fn invalid_player(invalid_choice: InvalidPlayer) -> Self {
        let rnd: i32 = rand::random();
        let mut username = format!("Player {}", rnd);
        let mut email = format!("player{}@gmail.com", rnd);
        let mut password = format!("PasswordIsVeryStrong");

        match invalid_choice {
            InvalidPlayer::Username => username = format!("1"),
            InvalidPlayer::Password => password = format!("pswrd"),
            InvalidPlayer::Email => email = format!("mail"),
        }
        Self {
            username,
            email,
            password,
            real_name: format!("A new player"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct UpdatePlayer {
    #[validate(length(
        min = 4,
        max = 20,
        message = "Username must be between 3 and 20 characters"
    ))]
    pub username: Option<String>,
    pub real_name: Option<String>,
    pub biography: Option<String>,
    pub country: Option<String>,
    pub flair: Option<String>,
    pub location: Option<String>,
    pub fide_rating: Option<i32>,
    pub social_links: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DisplayPlayer {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub biography: Option<String>,
    pub country: Option<String>,
    pub flair: Option<String>,
    pub real_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatedPlayer {
    #[schema(value_type = String, format = "uuid")]
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub biography: Option<String>,
    pub country: Option<String>,
    pub flair: Option<String>,
    pub real_name: String,
    pub location: Option<String>,
    pub fide_rating: Option<i32>,
    pub social_links: Option<Vec<String>>,
}

impl From<Model> for UpdatedPlayer {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            biography: Some(value.biography),
            country: Some(value.country),
            flair: Some(value.flair),
            real_name: value.real_name,
            location: value.location,
            fide_rating: value.fide_rating,
            social_links: value.social_links,
        }
    }
}

impl From<Model> for DisplayPlayer {
    fn from(value: Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
            biography: Some(value.biography),
            country: Some(value.country),
            flair: Some(value.flair),
            real_name: value.real_name,
        }
    }
}
