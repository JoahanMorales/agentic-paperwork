use serde::Serialize;

use crate::{config::Config, error::ApiError};

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: Vec<&'a str>,
    subject: &'a str,
    html: &'a str,
}

pub async fn send_email(
    http: &reqwest::Client,
    config: &Config,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), ApiError> {
    let api_key = config
        .resend_api_key
        .as_deref()
        .ok_or_else(|| ApiError::ServiceNotConfigured("RESEND_API_KEY".to_string()))?;
    let from = config
        .mail_from
        .as_deref()
        .ok_or_else(|| ApiError::ServiceNotConfigured("MAIL_FROM".to_string()))?;

    let response = http
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&ResendEmail {
            from,
            to: vec![to],
            subject,
            html,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError::ExternalService(format!(
            "Resend respondió {}: {}",
            status, text
        )));
    }

    Ok(())
}
