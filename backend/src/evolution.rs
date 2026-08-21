use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct TextMessage {
    text: String,
}

#[derive(Serialize)]
struct SendMessagePayload {
    number: String,
    text: String,
    #[serde(rename = "textMessage")]
    text_message: TextMessage,
    options: MessageOptions,
}

#[derive(Serialize)]
struct MessageOptions {
    delay: u32,
    presence: String,
}

#[derive(Serialize)]
struct CreateInstancePayload {
    #[serde(rename = "instanceName")]
    instance_name: String,
    token: String,
    qrcode: bool,
    integration: String,
}

#[derive(Deserialize, Debug)]
pub struct EvolutionResponse {
    pub key: Option<MessageKey>,
    pub error: Option<String>,
    pub message: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct MessageKey {
    pub id: String,
}

#[derive(Clone)]
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
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Formata o telefone garantindo código do país (55 para Brasil).
    pub fn normalize_phone_number(raw: &str) -> String {
        let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.starts_with("55") && (digits.len() == 12 || digits.len() == 13) {
            digits
        } else if digits.len() == 10 || digits.len() == 11 {
            format!("55{}", digits)
        } else {
            digits
        }
    }

    /// Cria uma nova instância na Evolution API (se ainda não existir).
    pub async fn create_instance(
        &self,
        instance_name: &str,
        api_key: &str,
    ) -> Result<Option<String>, String> {
        let endpoint = format!("{}/instance/create", self.base_url);

        let payload = CreateInstancePayload {
            instance_name: instance_name.to_string(),
            token: instance_name.to_string(),
            qrcode: true,
            integration: "WHATSAPP-BAILEYS".to_string(),
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("apikey", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Falha de conexão com Evolution API (create): {}", e))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|_| "Falha ao processar resposta do Evolution API.".to_string())?;

        if status.is_success() {
            // Tenta extrair QR code da criação se vier
            let qr = body
                .get("qrcode")
                .and_then(|q| q.get("base64"))
                .and_then(|b| b.as_str())
                .or_else(|| body.get("base64").and_then(|b| b.as_str()))
                .map(|s| s.to_string());
            Ok(qr)
        } else if status.as_u16() == 403 || status.as_u16() == 400 {
            // Instância provavelmente já existe, segue para conexão
            Ok(None)
        } else {
            let msg = body
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Erro ao criar instância");
            Err(format!("Evolution API Erro ({}): {}", status, msg))
        }
    }

    /// Conecta à instância existente e obtém o QR Code em base64.
    pub async fn connect_instance(
        &self,
        instance_name: &str,
        api_key: &str,
    ) -> Result<String, String> {
        let endpoint = format!("{}/instance/connect/{}", self.base_url, instance_name);

        let response = self
            .http_client
            .get(&endpoint)
            .header("apikey", api_key)
            .send()
            .await
            .map_err(|e| format!("Falha ao conectar à Evolution API: {}", e))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|_| "Falha ao ler dados de conexão do WhatsApp.".to_string())?;

        if status.is_success() {
            let qr_opt = body
                .get("base64")
                .and_then(|b| b.as_str())
                .or_else(|| {
                    body.get("qrcode")
                        .and_then(|q| q.get("base64"))
                        .and_then(|b| b.as_str())
                })
                .or_else(|| body.get("code").and_then(|b| b.as_str()))
                .or_else(|| {
                    body.get("qrcode")
                        .and_then(|q| q.get("code"))
                        .and_then(|b| b.as_str())
                });

            if let Some(qr) = qr_opt {
                Ok(qr.to_string())
            } else {
                // Pode ser que a instância já esteja conectada
                if let Some(state) = body.get("state").and_then(|s| s.as_str()) {
                    if state == "open" {
                        return Ok("ALREADY_CONNECTED".to_string());
                    }
                }
                Err("QR Code não retornado pela Evolution API. Verifique o status da instância.".to_string())
            }
        } else {
            let err_text = body.to_string();
            Err(format!("Erro ao gerar QR Code ({}): {}", status, err_text))
        }
    }

    /// Consulta o estado da conexão ("open", "connecting", "close", "disconnected").
    pub async fn get_connection_state(
        &self,
        instance_name: &str,
        api_key: &str,
    ) -> Result<String, String> {
        let endpoint = format!("{}/instance/connectionState/{}", self.base_url, instance_name);

        let response = self
            .http_client
            .get(&endpoint)
            .header("apikey", api_key)
            .send()
            .await
            .map_err(|e| format!("Falha ao checar status da Evolution API: {}", e))?;

        let status = response.status();
        if status.is_success() {
            let body: serde_json::Value = response
                .json()
                .await
                .map_err(|_| "Falha ao ler status da conexão.".to_string())?;

            let state = body
                .get("instance")
                .and_then(|i| i.get("state"))
                .and_then(|s| s.as_str())
                .or_else(|| body.get("state").and_then(|s| s.as_str()))
                .unwrap_or("close");

            Ok(state.to_string())
        } else {
            Ok("disconnected".to_string())
        }
    }

    /// Desconecta a sessão do WhatsApp na Evolution API.
    pub async fn disconnect_instance(
        &self,
        instance_name: &str,
        api_key: &str,
    ) -> Result<(), String> {
        let endpoint = format!("{}/instance/logout/{}", self.base_url, instance_name);

        let response = self
            .http_client
            .delete(&endpoint)
            .header("apikey", api_key)
            .send()
            .await
            .map_err(|e| format!("Falha ao desconectar da Evolution API: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Erro ao desconectar ({}): {}", status, error_text))
        }
    }

    /// Envia mensagem de texto via WhatsApp suportando v1 e v2 da Evolution API.
    pub async fn send_whatsapp_text(
        &self,
        instance_name: &str,
        api_key: &str,
        phone_number: &str,
        message: &str,
    ) -> Result<String, String> {
        let endpoint = format!("{}/message/sendText/{}", self.base_url, instance_name);
        let formatted_number = Self::normalize_phone_number(phone_number);

        let payload = SendMessagePayload {
            number: formatted_number,
            text: message.to_string(),
            text_message: TextMessage {
                text: message.to_string(),
            },
            options: MessageOptions {
                delay: 1200,
                presence: "composing".to_string(),
            },
        };

        let response = self
            .http_client
            .post(&endpoint)
            .header("apikey", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Falha ao comunicar com a Evolution API: {}", e))?;

        let status = response.status();
        if status.is_success() {
            let data: EvolutionResponse = response
                .json()
                .await
                .map_err(|_| "Falha ao processar resposta do envio de WhatsApp.".to_string())?;

            if let Some(err) = data.error {
                return Err(err);
            }

            let message_id = data.key.map(|k| k.id).unwrap_or_default();
            Ok(message_id)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Evolution API Erro ({}): {}", status, error_text))
        }
    }

    /// Exclui uma mensagem enviada (se necessário).
    pub async fn delete_whatsapp_message(
        &self,
        instance_name: &str,
        api_key: &str,
        phone_number: &str,
        message_id: &str,
    ) -> Result<(), String> {
        let endpoint = format!("{}/chat/deleteMessage/{}", self.base_url, instance_name);
        let formatted_number = Self::normalize_phone_number(phone_number);

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
            .map_err(|e| format!("Falha de conexão com Evolution API: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            Err(format!("Erro ao excluir mensagem ({}): {}", status, error_text))
        }
    }
}
