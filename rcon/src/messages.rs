use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A request sent to the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconRequest {
    #[serde(rename = "authToken")]
    pub auth_token: String,

    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "contentBody")]
    pub content_body: String,
}

impl RconRequest {
    /// Create a new request.
    pub fn new(name: impl Into<String>, content_body: impl Into<String>) -> Self {
        Self {
            auth_token: String::new(),
            name: name.into(),
            content_body: content_body.into(),
            version: "2".into(),
        }
    }

    /// Create a new request.
    pub fn with_body(name: impl Into<String>, content_body: Value) -> Self {
        Self {
            auth_token: String::new(),
            name: name.into(),
            content_body: serde_json::to_string(&content_body).unwrap(),
            version: "2".into(),
        }
    }

    /// Serialize this request into bytes.
    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_string(self).unwrap().bytes().collect()
    }
}

/// A response received back from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RconResponse {
    #[serde(rename = "statusCode")]
    pub status_code: i32,

    #[serde(rename = "statusMessage")]
    pub status_message: String,

    #[serde(rename = "version")]
    pub version: i32,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "contentBody")]
    pub content_body: String,
}

impl RconResponse {
    /// Assert that the status code is 200 otherwise return [`T`].
    pub fn assert_ok<T>(&self, err: T) -> Result<(), T> {
        match self.status_code {
            200 => Ok(()),
            _ => Err(err),
        }
    }
}
