use std::time::Instant;

use serde::Serialize;
use tracing::{error, info};

use crate::config::{AppConfig, AuthPerms};

#[derive(Debug, Clone, Serialize)]
pub struct AuthHandle {
    #[serde(skip_serializing)]
    pub granted_at: Instant,

    pub name: String,
    pub perms: AuthPerms,
}

impl AuthHandle {
    pub fn default_no_perms() -> Self {
        AuthHandle {
            granted_at: Instant::now(),
            name: "default-token".to_string(),
            perms: AuthPerms::default(),
        }
    }
}

pub fn authenticate_token(provided_token: &str, config: &AppConfig) -> Result<AuthHandle, ()> {
    let cnf = config.borrow();
    let matched_token = cnf
        .auth
        .tokens
        .iter()
        .find(|t| t.value == provided_token)
        .clone();

    if matched_token.is_none() {
        error!(
            "Client attempted to authenticate with non-existent token {}",
            provided_token
        );
        return Err(());
    }
    let matched_token = matched_token.unwrap();

    let handle = AuthHandle {
        granted_at: Instant::now(),
        name: matched_token.name.clone(),
        perms: matched_token.perms.clone(),
    };

    info!("Granted handle {:?}", handle);
    Ok(handle)
}
