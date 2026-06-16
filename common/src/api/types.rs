use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse<T: Serialize> {
    Success(T),
    Error { message: String },
}
