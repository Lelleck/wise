use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebSocketError {
    #[error("The provided password {0:?} is incorrect.")]
    InvalidPassword(Option<String>),

    #[error("The other side failed to provide a correct password.")]
    PasswordTimeout,
}
