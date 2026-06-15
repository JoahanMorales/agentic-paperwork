use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("recurso no encontrado")]
    NotFound,
    #[error("solicitud inválida: {0}")]
    BadRequest(String),
    #[error("no autorizado")]
    Unauthorized,
    #[error("servicio externo no configurado: {0}")]
    ServiceNotConfigured(String),
    #[error("error de servicio externo: {0}")]
    ExternalService(String),
    #[error("error de base de datos: {0}")]
    Db(#[from] sqlx::Error),
    #[error("error HTTP: {0}")]
    Http(#[from] reqwest::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::ServiceNotConfigured(_) => StatusCode::PRECONDITION_REQUIRED,
            Self::ExternalService(_) => StatusCode::BAD_GATEWAY,
            Self::Db(_) | Self::Http(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string()
        }))
    }
}
