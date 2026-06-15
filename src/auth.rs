use actix_web::HttpRequest;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, Clone)]
pub struct Actor {
    pub id: Option<Uuid>,
    pub rol: Role,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Publico,
    Cliente,
    Cajero,
    Administrador,
    Propietario,
}

impl Role {
    pub fn is_staff(&self) -> bool {
        matches!(self, Self::Cajero | Self::Administrador | Self::Propietario)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Administrador | Self::Propietario)
    }
}

pub fn actor_from_headers(req: &HttpRequest) -> Result<Actor, ApiError> {
    let rol = req
        .headers()
        .get("x-role")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("publico");

    let rol = match rol {
        "cliente" => Role::Cliente,
        "cajero" => Role::Cajero,
        "administrador" => Role::Administrador,
        "propietario" => Role::Propietario,
        "publico" => Role::Publico,
        _ => return Err(ApiError::Unauthorized),
    };

    let id = req
        .headers()
        .get("x-user-id")
        .and_then(|h| h.to_str().ok())
        .and_then(|raw| Uuid::parse_str(raw).ok());

    Ok(Actor { id, rol })
}

pub fn require_staff(req: &HttpRequest) -> Result<Actor, ApiError> {
    let actor = actor_from_headers(req)?;
    actor
        .rol
        .is_staff()
        .then_some(actor)
        .ok_or(ApiError::Unauthorized)
}

pub fn require_admin(req: &HttpRequest) -> Result<Actor, ApiError> {
    let actor = actor_from_headers(req)?;
    actor
        .rol
        .is_admin()
        .then_some(actor)
        .ok_or(ApiError::Unauthorized)
}

pub fn require_cliente(req: &HttpRequest) -> Result<Actor, ApiError> {
    let actor = actor_from_headers(req)?;
    (actor.rol == Role::Cliente && actor.id.is_some())
        .then_some(actor)
        .ok_or(ApiError::Unauthorized)
}
