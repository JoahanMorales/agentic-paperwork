use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

pub async fn write_audit(
    pool: &PgPool,
    usuario_id: Option<Uuid>,
    accion: &str,
    entidad: &str,
    entidad_id: Option<Uuid>,
    anteriores: Option<Value>,
    nuevos: Option<Value>,
) -> Result<(), ApiError> {
    sqlx::query(
        "insert into log_auditoria (usuario_id, accion, entidad_afectada, entidad_id, valores_anteriores, valores_nuevos)
         values ($1,$2,$3,$4,$5,$6)",
    )
    .bind(usuario_id)
    .bind(accion)
    .bind(entidad)
    .bind(entidad_id)
    .bind(anteriores)
    .bind(nuevos)
    .execute(pool)
    .await?;

    Ok(())
}
