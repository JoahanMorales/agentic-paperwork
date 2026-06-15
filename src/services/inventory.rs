use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Serialize)]
pub struct InventoryRunResult {
    pub alertas_generadas: i32,
    pub ordenes_generadas: i32,
}

pub async fn run_inventory_agent(
    pool: &PgPool,
    producto_id: Option<Uuid>,
) -> Result<InventoryRunResult, ApiError> {
    let rows = sqlx::query(
        "select p.id, p.nombre, p.stock_actual, p.punto_reorden, p.precio_costo, p.proveedor_principal_id,
                pr.canal_digital, pr.tiene_orden_previa_exitosa
         from productos p
         left join proveedores pr on pr.id = p.proveedor_principal_id
         where p.estado <> 'inactivo'
           and p.stock_actual <= p.punto_reorden
           and ($1::uuid is null or p.id = $1)",
    )
    .bind(producto_id)
    .fetch_all(pool)
    .await?;

    let mut alertas = 0;
    let mut ordenes = 0;

    for row in rows {
        let product_id: Uuid = row.get("id");
        let nombre: String = row.get("nombre");
        let stock: i32 = row.get("stock_actual");
        let punto: i32 = row.get("punto_reorden");
        let proveedor_id: Option<Uuid> = row.get("proveedor_principal_id");

        sqlx::query(
            "insert into alertas (tipo, descripcion, producto_id)
             values ('reorden', $1, $2)",
        )
        .bind(format!(
            "{nombre} alcanzó punto de reorden: stock {stock} <= {punto}"
        ))
        .bind(product_id)
        .execute(pool)
        .await?;
        alertas += 1;

        if let Some(proveedor_id) = proveedor_id {
            let cantidad = (punto * 2 - stock).max(1);
            let costo: Decimal = row.get("precio_costo");
            let monto = costo * Decimal::from(cantidad);
            let canal_digital: Option<bool> = row.get("canal_digital");
            let previa_exitosa: Option<bool> = row.get("tiene_orden_previa_exitosa");
            let puede_enviar = canal_digital.unwrap_or(false) && previa_exitosa.unwrap_or(false);
            let requiere_aprobacion = !puede_enviar || monto > Decimal::from(5000);
            let estado = if requiere_aprobacion {
                "pendiente_aprobacion"
            } else {
                "enviada"
            };

            let order_id: Uuid = sqlx::query(
                "insert into ordenes_reposicion (proveedor_id, estado, monto_total, generada_por, requiere_aprobacion, fecha_envio)
                 values ($1,$2,$3,'agente_ia',$4, case when $2 = 'enviada' then now() else null end)
                 returning id",
            )
            .bind(proveedor_id)
            .bind(estado)
            .bind(monto)
            .bind(requiere_aprobacion)
            .fetch_one(pool)
            .await?
            .get("id");

            sqlx::query(
                "insert into detalle_orden_reposicion (orden_id, producto_id, cantidad, precio_unitario_costo)
                 values ($1,$2,$3,$4)",
            )
            .bind(order_id)
            .bind(product_id)
            .bind(cantidad)
            .bind(costo)
            .execute(pool)
            .await?;
            ordenes += 1;
        }
    }

    Ok(InventoryRunResult {
        alertas_generadas: alertas,
        ordenes_generadas: ordenes,
    })
}
