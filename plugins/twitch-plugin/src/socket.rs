use futures_util::StreamExt;
use reqwest::Client;
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use twitch_api::{
    HelixClient,
    eventsub::{Transport, channel::ChannelChatMessageEventSubTopic},
    helix::{chat::SendChatMessageRequest, eventsub::CreateEventSubSubscriptionBody},
    twitch_oauth2::UserToken,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Client::new();
    let helix_client = HelixClient::new();

    // 1. Dein Token (muss Scopes 'user:read:chat' und 'user:write:chat' haben)
    let mut user_token = get_user_token_from_db().await?;
    let broadcaster_id = "123456"; // Die ID des Streamers, dessen Chat du lesen willst
    let bot_user_id = "987654"; // Deine eigene User-ID (bzw. die des Bots)

    // 2. Mit dem Twitch EventSub WebSocket verbinden
    let url = "wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=10";
    println!("Verbinde mit Twitch EventSub...");
    let (ws_stream, _) = connect_async(url).await?;
    let (_, mut read) = ws_stream.split();

    // 3. Die WebSocket-Event-Loop starten
    while let Some(message) = read.next().await {
        let msg = match message {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => {
                println!("Verbindung von Twitch geschlossen.");
                break;
            }
            _ => continue,
        };

        // JSON parsen
        let json: Value = serde_json::from_str(&msg)?;

        // Twitch schickt als erstes eine "session_welcome" Nachricht
        if let Some(message_type) = json.get("metadata").and_then(|m| m.get("message_type")) {
            if message_type == "session_welcome" {
                // Session ID extrahieren
                let session_id = json["payload"]["session"]["id"].as_str().unwrap();
                println!("Erfolgreich verbunden! Session ID: {}", session_id);

                // JETZT ERST: Den Chat für diese Session abonnieren via Helix
                aboniere_chat_events(
                    &helix_client,
                    &http_client,
                    &user_token,
                    broadcaster_id,
                    bot_user_id,
                    session_id,
                )
                .await?;

                println!("Chat-Abonnement aktiv. Warte auf Nachrichten...");
            }

            // Wenn eine echte Chat-Nachricht reinkommt
            if message_type == "notification" {
                let event = &json["payload"]["event"];
                let text = event["message"]["text"].as_str().unwrap_or("");
                let user_name = event["chatter_user_name"].as_str().unwrap_or("Unbekannt");
                let message_id = event["message_id"].as_str().unwrap_or("");

                println!("[{}] schrieb: {}", user_name, text);

                // Auf "!ping" reagieren
                if text.starts_with("!ping") {
                    // Token-Refresh-Check vor dem Senden
                    if !user_token.validate_token(&http_client).await {
                        user_token
                            .refresh_token(&http_client)
                            .bind(&user_token.client_id, &user_token.client_secret)
                            .await?;
                    }

                    println!("Sende Pong...");
                    let reply =
                        SendChatMessageRequest::new(broadcaster_id, bot_user_id, "Pong! 🏓")
                            .reply_parent_message_id(Some(message_id.into()));

                    helix_client
                        .req_post(reply, &http_client, &user_token)
                        .await?;
                }
            }
        }
    }

    Ok(())
}

// Hilfsfunktion: Sagt Twitch via Helix, dass die Nachrichten an unseren WebSocket geleitet werden sollen
async fn aboniere_chat_events(
    helix_client: &HelixClient<'static, Client>,
    http_client: &Client,
    token: &UserToken,
    broadcaster_id: &str,
    bot_user_id: &str,
    session_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Wir definieren WAS wir hören wollen (Chat-Nachrichten)
    let topic = ChannelChatMessageEventSubTopic::new(broadcaster_id, bot_user_id);

    // Wir definieren WOHIN es geschickt werden soll (Unsere WebSocket Session)
    let transport = Transport::websocket(session_id.to_string());

    let body = CreateEventSubSubscriptionBody::new(topic, transport);

    // Der Helix-Call, der das Abo aktiviert
    helix_client
        .create_eventsub_subscription(body, token, http_client)
        .await?;

    Ok(())
}
