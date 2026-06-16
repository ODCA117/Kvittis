use crate::{NewUser, User};
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
    pub token_type: TokenType,
}

// Request enums

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UnauthorizedUserRequest {
    Register { user: NewUser },
    Login { username: String, password: String },
}

// Shared types

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
    Bearer,
}

// From trait implementations

impl From<User> for RegisterResponse {
    fn from(value: User) -> Self {
        RegisterResponse { user: value }
    }
}

impl From<RegisterResponse> for User {
    fn from(value: RegisterResponse) -> Self {
        value.user
    }
}
