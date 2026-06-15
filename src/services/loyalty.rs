use chrono::{Duration, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

pub async fn acreditar_puntos(
    pool: &PgPool,
    cliente_id: Uuid,
    pedido_id: Uuid,
    total: Decimal,
) -> Result<i32, ApiError> {
    let puntos = total.to_i32().unwrap_or(0) / 10;
    if puntos <= 0 {
        return Ok(0);
    }

    let vencimiento = Utc::now().date_naive() + Duration::days(365);
    sqlx::query(
        "insert into transacciones_puntos (cliente_id, pedido_id, tipo, puntos, fecha_vencimiento)
         values ($1,$2,'acumulacion',$3,$4)",
    )
    .bind(cliente_id)
    .bind(pedido_id)
    .bind(puntos)
    .bind(vencimiento)
    .execute(pool)
    .await?;

    sqlx::query(
        "update clientes set es_cliente_frecuente = true, fecha_clasificacion_frecuente = current_date
         where id = $1 and (
           select count(*) from pedidos
           where cliente_id = $1 and created_at >= now() - interval '3 months' and estado in ('pagado','completado')
         ) >= 5",
    )
    .bind(cliente_id)
    .execute(pool)
    .await?;

    Ok(puntos)
}
