use super::keychain;

use twitch_api::TwitchClient;
use twitch_api::twitch_oauth2::{ClientId, DeviceUserTokenBuilder, Scope};

pub async fn run(client_id: ClientId) -> Result<(), Box<dyn std::error::Error>> {
    let client: TwitchClient<reqwest::Client> = TwitchClient::default();
    let mut builder = DeviceUserTokenBuilder::new(client_id, self::admin_scopes()); // fixme different scopes for bot user!

    let code = builder.start(&client).await?;
    println!("Please go to {}", code.verification_uri);

    let token = builder.wait_for_code(&client, tokio::time::sleep).await?;
    keychain::store_user_token(&token)?;

    Ok(())
}

pub fn admin_scopes() -> Vec<Scope> {
    vec![
        Scope::ChatRead,
        Scope::ChatEdit,
        Scope::UserReadChat,
        Scope::UserWriteChat,
    ]
}

// todo
pub fn _bot_scopes() -> Vec<Scope> {
    vec![
        Scope::ChatRead,
        Scope::ChatEdit,
        Scope::UserReadChat,
        Scope::UserWriteChat,
    ]
}
