//! Firepass/Kimi LLM client implementation.

use super::{LlmClient, LlmError, TechCard};
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

/// Firepass-backed Kimi client.
pub struct FirepassKimiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: SecretBox<String>,
}

impl FirepassKimiClient {
    /// Build from `KIMI_API_KEY` and `FIREPASS_BASE_URL`.
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("KIMI_API_KEY").map_err(|_| LlmError::NetworkUnavailable)?;
        let base_url = std::env::var("FIREPASS_BASE_URL")
            .unwrap_or_else(|_| "https://api.firepass.dev/v1".to_string());
        Ok(Self {
            client: reqwest::Client::new(),
            base_url,
            api_key: SecretBox::new(Box::new(api_key)),
        })
    }

    fn extract_content(response: ChatResponse) -> Result<String, LlmError> {
        response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| LlmError::InvalidResponse("missing chat completion content".into()))
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<MessageReq<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Debug, serde::Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, serde::Serialize)]
struct MessageReq<'a> {
    role: &'a str,
    content: &'a str,
}

impl LlmClient for FirepassKimiClient {
    async fn propose_tech_card(
        &self,
        prompt: &str,
        snapshot_hash: &[u8],
    ) -> Result<TechCard, LlmError> {
        let content = format!(
            "{prompt}\n\nSnapshot hash: {snapshot_hash:?}\nReturn only a JSON tech-card with fields id, era, inputs, energy_cost, byproducts, dependencies."
        );
        let request = ChatCompletionRequest {
            model: "kimi-k2.6-turbo",
            messages: vec![MessageReq {
                role: "user",
                content: &content,
            }],
            response_format: Some(ResponseFormat {
                kind: "json_object",
            }),
            max_tokens: None,
            temperature: None,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(url)
            .bearer_auth(self.api_key.expose_secret())
            .json(&request)
            .send()
            .await
            .map_err(|_| LlmError::NetworkUnavailable)?;

        let response = response.error_for_status().map_err(|error| {
            if error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                LlmError::RateLimited
            } else {
                LlmError::NetworkUnavailable
            }
        })?;

        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|_| LlmError::InvalidResponse("invalid response envelope".into()))?;
        let content = Self::extract_content(chat)?;
        serde_json::from_str(&content)
            .map_err(|_| LlmError::InvalidResponse("invalid tech-card json".into()))
    }

    /// Generate generic text through the same authenticated chat-completions
    /// path used by tech-card proposals.
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
        json_schema: Option<&str>,
    ) -> Result<String, LlmError> {
        let prompt = match json_schema {
            Some(schema) => format!("{prompt}\n\nReturn only JSON matching this schema:\n{schema}"),
            None => prompt.to_string(),
        };
        let request = ChatCompletionRequest {
            model: "kimi-k2.6-turbo",
            messages: vec![MessageReq {
                role: "user",
                content: &prompt,
            }],
            response_format: json_schema.map(|_| ResponseFormat {
                kind: "json_object",
            }),
            max_tokens: Some(max_tokens.max(1)),
            temperature: Some(temperature.max(0.0)),
        };
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let response = self
            .client
            .post(url)
            .bearer_auth(self.api_key.expose_secret())
            .json(&request)
            .send()
            .await
            .map_err(|_| LlmError::NetworkUnavailable)?;
        let response = response.error_for_status().map_err(|error| {
            if error.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                LlmError::RateLimited
            } else {
                LlmError::NetworkUnavailable
            }
        })?;
        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|_| LlmError::InvalidResponse("invalid response envelope".into()))?;
        Self::extract_content(chat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    /// Serializes env-var mutation; parallel tests would race on `KIMI_API_KEY`.
    fn env_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn from_env_requires_api_key() {
        let _guard = env_lock();
        std::env::remove_var("KIMI_API_KEY");
        std::env::remove_var("FIREPASS_BASE_URL");
        assert!(matches!(
            FirepassKimiClient::from_env(),
            Err(LlmError::NetworkUnavailable)
        ));
    }

    #[test]
    fn from_env_uses_default_base_url() {
        let _guard = env_lock();
        std::env::set_var("KIMI_API_KEY", "test-key");
        std::env::remove_var("FIREPASS_BASE_URL");
        let client = FirepassKimiClient::from_env().expect("client");
        assert_eq!(client.base_url, "https://api.firepass.dev/v1");
        assert_eq!(client.api_key.expose_secret(), "test-key");
        std::env::remove_var("KIMI_API_KEY");
    }

    #[test]
    fn from_env_happy_path_with_explicit_base_url() {
        let _guard = env_lock();
        std::env::set_var("KIMI_API_KEY", "test-key");
        std::env::set_var("FIREPASS_BASE_URL", "https://example.invalid/v1");
        let client = FirepassKimiClient::from_env().expect("client");
        assert_eq!(client.base_url, "https://example.invalid/v1");
        assert_eq!(client.api_key.expose_secret(), "test-key");
        std::env::remove_var("KIMI_API_KEY");
        std::env::remove_var("FIREPASS_BASE_URL");
    }

    #[test]
    fn generic_request_omits_json_mode_for_plain_text() {
        let request = ChatCompletionRequest {
            model: "kimi-k2.6-turbo",
            messages: vec![MessageReq {
                role: "user",
                content: "hello",
            }],
            response_format: None,
            max_tokens: Some(32),
            temperature: Some(0.2),
        };
        let value = serde_json::to_value(request).expect("request json");
        assert!(value.get("response_format").is_none());
        assert_eq!(value["max_tokens"], 32);
    }
}
