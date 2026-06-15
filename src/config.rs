use std::{env, str::FromStr};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,

    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
    pub openrouter_site_url: Option<String>,
    pub openrouter_app_name: String,
    pub resend_api_key: Option<String>,
    pub mail_from: Option<String>,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/papermind".to_string()
            }),
            supabase_url: env::var("SUPABASE_URL").ok(),
            supabase_anon_key: env::var("SUPABASE_ANON_KEY").ok(),

            openrouter_api_key: env::var("OPENROUTER_API_KEY").ok(),
            openrouter_model: env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "openai/gpt-4o-mini".to_string()),
            openrouter_site_url: env::var("OPENROUTER_SITE_URL").ok(),
            openrouter_app_name: env::var("OPENROUTER_APP_NAME")
                .unwrap_or_else(|_| "PaperMind".to_string()),
            resend_api_key: env::var("RESEND_API_KEY").ok(),
            mail_from: env::var("MAIL_FROM").ok(),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| u16::from_str(&p).ok())
                .unwrap_or(8080),
        }
    }

    pub fn openrouter_enabled(&self) -> bool {
        self.openrouter_api_key.is_some()
    }

    pub fn mail_enabled(&self) -> bool {
        self.resend_api_key.is_some() && self.mail_from.is_some()
    }
}
