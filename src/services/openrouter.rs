use serde::{Deserialize, Serialize};

use crate::{config::Config, error::ApiError};

#[derive(Serialize)]
struct OpenRouterRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    max_tokens: u16,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

pub async fn chat_completion(
    http: &reqwest::Client,
    config: &Config,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, ApiError> {
    let api_key = config
        .openrouter_api_key
        .as_deref()
        .ok_or_else(|| ApiError::ServiceNotConfigured("OPENROUTER_API_KEY".to_string()))?;

    let mut request = http
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&OpenRouterRequest {
            model: &config.openrouter_model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: user_prompt,
                },
            ],
            temperature: 0.3,
            max_tokens: 700,
        });

    if let Some(site_url) = &config.openrouter_site_url {
        request = request.header("HTTP-Referer", site_url);
    }
    request = request.header("X-Title", &config.openrouter_app_name);

    let response = request.send().await?;
    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(ApiError::ExternalService(format!(
            "OpenRouter respondió {}: {}",
            status, text
        )));
    }

    let parsed: OpenRouterResponse = serde_json::from_str(&text)
        .map_err(|e| ApiError::ExternalService(format!("respuesta inválida de OpenRouter: {e}")))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .ok_or_else(|| ApiError::ExternalService("OpenRouter no devolvió respuesta".to_string()))
}
