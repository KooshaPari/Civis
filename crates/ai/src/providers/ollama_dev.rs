//! `OllamaDevProvider` — dev-only, OpenAI-compatible HTTP (FR-CIV-AI-003).
//!
//! Reuses the same chat-completions client shape as the cloud provider.
//! **Never** a shipping dependency — behind the `dev` feature only.
//!
//! The provider uses the OpenAI-compatible chat-completions contract and
//! remains behind the `dev` feature so it cannot enter a shipping build.

use crate::{AiError, AiProvider, Capabilities, EmbedRequest, GenOutput, GenRequest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: [Message<'a>; 1],
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

/// Dev-only Ollama provider using an OpenAI-compatible HTTP endpoint.
pub struct OllamaDevProvider {
    client: reqwest::Client,
    endpoint: String,
    model_id: String,
}

impl OllamaDevProvider {
    /// Build with the Ollama endpoint + model id.
    #[must_use]
    pub fn new(endpoint: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: endpoint.into(),
            model_id: model_id.into(),
        }
    }

    fn chat_url(&self) -> String {
        let endpoint = self.endpoint.trim_end_matches('/');
        if endpoint.ends_with("/chat/completions") {
            endpoint.to_string()
        } else if endpoint.ends_with("/v1") {
            format!("{}/chat/completions", endpoint)
        } else {
            format!("{}/v1/chat/completions", endpoint)
        }
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaDevProvider {
    async fn generate(&self, req: &GenRequest) -> Result<GenOutput, AiError> {
        let body = ChatRequest {
            model: &self.model_id,
            messages: [Message {
                role: "user",
                content: &req.prompt,
            }],
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            response_format: req.json_schema.as_ref().map(|_| ResponseFormat {
                kind: "json_object",
            }),
        };
        let response = self
            .client
            .post(self.chat_url())
            .json(&body)
            .send()
            .await
            .map_err(|err| AiError::Unavailable(format!("Ollama request failed: {err}")))?;
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited);
        }
        if !response.status().is_success() {
            return Err(AiError::Unavailable(format!(
                "Ollama returned HTTP {}",
                response.status()
            )));
        }
        let payload = response.json::<ChatResponse>().await.map_err(|err| {
            AiError::InvalidResponse(format!("Ollama response JSON decode failed: {err}"))
        })?;
        let text = payload
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| AiError::InvalidResponse("Ollama response had no content".into()))?;
        Ok(GenOutput::fresh(text))
    }

    async fn embed(&self, _req: &EmbedRequest) -> Result<Vec<Vec<f32>>, AiError> {
        Err(AiError::Unsupported("ollama-dev".into()))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn model_version(&self) -> &str {
        "dev"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            generate: true,
            embed: false,
            cloud: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_normalization_targets_openai_chat_route() {
        assert_eq!(
            OllamaDevProvider::new("http://localhost:11434", "model").chat_url(),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            OllamaDevProvider::new("http://localhost:11434/v1/", "model").chat_url(),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            OllamaDevProvider::new("http://localhost:11434/v1/chat/completions", "model")
                .chat_url(),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn response_content_deserializes_from_openai_shape() {
        let response: ChatResponse =
            serde_json::from_str(r#"{"choices":[{"message":{"content":"hello"}}]}"#)
                .expect("valid chat response");
        assert_eq!(
            response.choices[0].message.content.as_deref(),
            Some("hello")
        );
    }
}
