use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use twitch_api::twitch_oauth2::TwitchToken;
use twitch_api::twitch_oauth2::{ClientId, RefreshToken, UserToken};

use crate::keychain;

// Wir bauen eine saubere Struktur, die du klonen und in jeden Thread/Task werfen kannst.
#[derive(Clone)]
pub struct Token {
    token: Arc<RwLock<UserToken>>,
    http_client: Client,
}

impl Token {
    pub async fn from_user(username: &str, client_id: ClientId) -> Option<Self> {
        let token_str = match keychain::get_token(username) {
            Ok(value) => value,
            Err(_) => {
                return None;
            }
        };

        let refresh_token = RefreshToken::from(token_str);
        let client = reqwest::Client::new();
        let token =
            match UserToken::from_refresh_token(&client.clone(), refresh_token, client_id, None)
                .await
            {
                Ok(value) => value,
                Err(_) => {
                    return None;
                }
            };

        keychain::store_user_token(&token.clone()).unwrap();

        return Some(Self::new(token));
    }

    pub fn new(token: UserToken) -> Self {
        Self {
            token: Arc::new(RwLock::new(token)),
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn get_token(&self) -> UserToken {
        let lock = self.token.read().await;
        if lock.is_elapsed() {
            self.refresh_token().await.unwrap();
        }

        lock.clone()
    }

    async fn refresh_token(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut lock = self.token.write().await;

        if lock.is_elapsed() {
            lock.refresh_token(&self.http_client).await?;
            keychain::store_user_token(&lock.clone()).unwrap();
        }

        Ok(())
    }
}
