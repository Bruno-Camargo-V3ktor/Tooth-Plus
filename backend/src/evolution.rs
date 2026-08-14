use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TextMessage {
    text: String,
}

#[derive(Serialize)]
struct SendMessagePayload {
    number: String,
    options: MessageOptions,
    #[serde(rename = "textMessage")]
    text_message: TextMessage,
}

#[derive(Serialize)]
struct MessageOptions {
    delay: u32,
}

#[derive(Deserialize, Debug)]
pub struct EvolutionResponse {
    pub key: Option<MessageKey>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct MessageKey {
    pub id: String,
}

pub struct EvolutionClient {
    base_url: String,
    http_client: Client,
}

#[derive(Serialize)]
struct DeleteMessagePayload {
    number: String,
    #[serde(rename = "messageId")]
    message_id: String,
}

impl EvolutionClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http_client: Client::new(),
        }
    }

    pub async fn send_whatsapp_text(
        &self,
        instance_name: &str,
        api_key: &str,
        phone_number: &str,
        message: &str,
    ) -> Result<String, String> {
        let endpoint = format!("{}/message/sendText/{}", self.base_url, instance_name);

        let formatted_number = phone_number.replace(|c: char| !c.is_numeric(), "");

        let payload = SendMessagePayload {
            number: formatted_number,
            options: MessageOptions { delay: 1200 },
            text_message: TextMessage {
                text: message.to_string(),
            },
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("apikey", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Evolution API: {}", e))?;

        if response.status().is_success() {
            let data: EvolutionResponse = response
                .json()
                .await
                .map_err(|_| "Failed to parse API response".to_string())?;

            if let Some(err) = data.error {
                return Err(err);
            }

            let message_id = data.key.map(|k| k.id).unwrap_or_default();
            Ok(message_id)
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("API Error ({}): {}", status, error_text))
        }
    }

    pub async fn delete_whatsapp_message(
        &self,
        instance_name: &str,
        api_key: &str,
        phone_number: &str,
        message_id: &str,
    ) -> Result<(), String> {
        let endpoint = format!("{}/chat/deleteMessage/{}", self.base_url, instance_name);
        let formatted_number = phone_number.replace(|c: char| !c.is_numeric(), "");

        let payload = DeleteMessagePayload {
            number: formatted_number,
            message_id: message_id.to_string(),
        };

        let response = self
            .http_client
            .delete(&endpoint)
            .header("apikey", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Evolution API: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("API Error ({}): {}", status, error_text))
        }
    }
}
