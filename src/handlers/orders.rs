use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, get, post,
    web::{Data, Json, Path},
};
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{Role, actor_from_headers, require_cliente, require_staff},
    error::ApiError,
    handlers::catalog::get_producto_simple,
    models::orders::{AgregarCarritoItem, ConfirmarPedido, CrearDevolucion, CrearVentaFisica},
    services::{
        audit::write_audit, inventory::run_inventory_agent, loyalty::acreditar_puntos,
        mail::send_email,
    },
    state::AppState,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(obtener_carrito)
        .service(agregar_carrito_item)
        .service(eliminar_carrito_item)
        .service(crear_pedido_desde_carrito)
        .service(listar_pedidos)
        .service(crear_venta_fisica)
        .service(crear_devolucion)
        .service(obtener_puntos);
}

#[get("/api/carrito")]
async fn obtener_carrito(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let actor = require_cliente(&req)?;
    let carrito_id = carrito_activo(&state.pool, actor.id.unwrap()).await?;
    Ok(HttpResponse::Ok().json(
        json!({"carrito_id": carrito_id, "items": carrito_items(&state.pool, carrito_id).await?}),
    ))
}

#[post("/api/carrito/items")]
async fn agregar_carrito_item(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<AgregarCarritoItem>,
) -> Result<impl Responder, ApiError> {
    let actor = require_cliente(&req)?;
    if body.cantidad <= 0 {
        return Err(ApiError::BadRequest(
            "la cantidad debe ser mayor a cero".into(),
        ));
    }
    let producto = get_producto_simple(&state.pool, body.producto_id).await?;
    if producto.stock_actual < body.cantidad {
        return Err(ApiError::BadRequest("no hay stock suficiente".into()));
    }

    let carrito_id = carrito_activo(&state.pool, actor.id.unwrap()).await?;
    sqlx::query(
        "insert into carrito_items (carrito_id, producto_id, cantidad) values ($1,$2,$3)
         on conflict (carrito_id, producto_id) do update set cantidad = carrito_items.cantidad + excluded.cantidad",
    )
    .bind(carrito_id)
    .bind(body.producto_id)
    .bind(body.cantidad)
    .execute(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(
        json!({"carrito_id": carrito_id, "items": carrito_items(&state.pool, carrito_id).await?}),
    ))
}

#[delete("/api/carrito/items/{producto_id}")]
async fn eliminar_carrito_item(
    req: HttpRequest,
    state: Data<AppState>,
    producto_id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let actor = require_cliente(&req)?;
    let carrito_id = carrito_activo(&state.pool, actor.id.unwrap()).await?;
    sqlx::query("delete from carrito_items where carrito_id = $1 and producto_id = $2")
        .bind(carrito_id)
        .bind(*producto_id)
        .execute(&state.pool)
        .await?;
    Ok(HttpResponse::Ok().json(json!({"mensaje": "producto eliminado del carrito"})))
}

#[post("/api/pedidos/desde-carrito")]
async fn crear_pedido_desde_carrito(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<ConfirmarPedido>,
) -> Result<impl Responder, ApiError> {
    let actor = require_cliente(&req)?;
    validar_metodo_pago(&body.metodo_pago)?;
    validar_modalidad(&body.modalidad_entrega)?;
    let cliente_id = actor.id.unwrap();
    let carrito_id = carrito_activo(&state.pool, cliente_id).await?;

    let mut tx = state.pool.begin().await?;
    let rows = sqlx::query(
        "select ci.producto_id, ci.cantidad, p.precio_venta, p.stock_actual, p.nombre
         from carrito_items ci join productos p on p.id = ci.producto_id
         where ci.carrito_id = $1",
    )
    .bind(carrito_id)
    .fetch_all(&mut *tx)
    .await?;

    if rows.is_empty() {
        return Err(ApiError::BadRequest("el carrito está vacío".into()));
    }

    let mut subtotal = Decimal::ZERO;
    for row in &rows {
        let cantidad: i32 = row.get("cantidad");
        let stock: i32 = row.get("stock_actual");
        if stock < cantidad {
            return Err(ApiError::BadRequest(format!(
                "stock insuficiente para {}",
                row.get::<String, _>("nombre")
            )));
        }
        subtotal += row.get::<Decimal, _>("precio_venta") * Decimal::from(cantidad);
    }

    let puntos_utilizados = body.puntos_utilizados.unwrap_or(0).max(0);
    let descuento_puntos = Decimal::from(puntos_utilizados / 100 * 5);
    let costo_envio = if body.modalidad_entrega == "domicilio" {
        Decimal::from(50)
    } else {
        Decimal::ZERO
    };
    let total = (subtotal - descuento_puntos + costo_envio).max(Decimal::ZERO);
    let estado = if body.metodo_pago == "oxxo" {
        "pendiente_pago"
    } else {
        "pagado"
    };
    let referencia_oxxo =
        (body.metodo_pago == "oxxo").then(|| format!("OXXO-{}", Uuid::new_v4().simple()));

    let pedido_id: Uuid = sqlx::query(
        "insert into pedidos (cliente_id, canal, estado, metodo_pago, modalidad_entrega, direccion_entrega,
                              subtotal, descuento_aplicado, costo_envio, puntos_utilizados, total,
                              referencia_oxxo, fecha_confirmacion)
         values ($1,'digital',$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,now()) returning id",
    )
    .bind(cliente_id)
    .bind(estado)
    .bind(&body.metodo_pago)
    .bind(&body.modalidad_entrega)
    .bind(&body.direccion_entrega)
    .bind(subtotal)
    .bind(descuento_puntos)
    .bind(costo_envio)
    .bind(puntos_utilizados)
    .bind(total)
    .bind(&referencia_oxxo)
    .fetch_one(&mut *tx)
    .await?
    .get("id");

    for row in &rows {
        let producto_id: Uuid = row.get("producto_id");
        let cantidad: i32 = row.get("cantidad");
        let precio: Decimal = row.get("precio_venta");
        sqlx::query("insert into detalle_pedido (pedido_id, producto_id, cantidad, precio_unitario) values ($1,$2,$3,$4)")
            .bind(pedido_id)
            .bind(producto_id)
            .bind(cantidad)
            .bind(precio)
            .execute(&mut *tx)
            .await?;

        if body.metodo_pago != "oxxo" {
            sqlx::query(
                "update productos set stock_actual = stock_actual - $1,
                 estado = case when stock_actual - $1 <= 0 then 'agotado' else estado end
                 where id = $2",
            )
            .bind(cantidad)
            .bind(producto_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    sqlx::query("insert into pagos (pedido_id, metodo, estado, referencia_pasarela, monto) values ($1,$2,$3,$4,$5)")
        .bind(pedido_id)
        .bind(&body.metodo_pago)
        .bind(if body.metodo_pago == "oxxo" { "pendiente" } else { "aprobado" })
        .bind(referencia_oxxo.as_deref().unwrap_or("simulado-pci-dss"))
        .bind(total)
        .execute(&mut *tx)
        .await?;
    sqlx::query("update carritos set estado = 'convertido' where id = $1")
        .bind(carrito_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    if body.metodo_pago != "oxxo" {
        acreditar_puntos(&state.pool, cliente_id, pedido_id, total).await?;
        run_inventory_agent(&state.pool, None).await?;
    }
    send_optional_order_email(&state, &body.email_confirmacion, pedido_id, total).await?;

    Ok(HttpResponse::Created().json(json!({
        "pedido_id": pedido_id,
        "estado": estado,
        "total": total,
        "referencia_oxxo": referencia_oxxo,
        "correo_enviado": body.email_confirmacion.is_some() && state.config.mail_enabled()
    })))
}

#[post("/api/pos/ventas")]
async fn crear_venta_fisica(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<CrearVentaFisica>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    validar_metodo_pago(&body.metodo_pago)?;
    if body.items.is_empty() {
        return Err(ApiError::BadRequest(
            "la venta requiere al menos un producto".into(),
        ));
    }

    let mut subtotal = Decimal::ZERO;
    let mut detalle = Vec::new();
    for item in &body.items {
        if item.cantidad <= 0 {
            return Err(ApiError::BadRequest("cantidad inválida".into()));
        }
        let p = get_producto_simple(&state.pool, item.producto_id).await?;
        if p.stock_actual < item.cantidad {
            return Err(ApiError::BadRequest(format!(
                "stock insuficiente para {}",
                p.nombre
            )));
        }
        subtotal += p.precio_venta * Decimal::from(item.cantidad);
        detalle.push((p.id, item.cantidad, p.precio_venta));
    }

    let mut tx = state.pool.begin().await?;
    let pedido_id: Uuid = sqlx::query(
        "insert into pedidos (cliente_id, usuario_sistema_id, canal, estado, metodo_pago, modalidad_entrega,
                              subtotal, total, fecha_confirmacion)
         values ($1,$2,'fisico','pagado',$3,'mostrador',$4,$4,now()) returning id",
    )
    .bind(body.cliente_id)
    .bind(body.usuario_sistema_id)
    .bind(&body.metodo_pago)
    .bind(subtotal)
    .fetch_one(&mut *tx)
    .await?
    .get("id");

    for (producto_id, cantidad, precio) in detalle {
        sqlx::query("insert into detalle_pedido (pedido_id, producto_id, cantidad, precio_unitario) values ($1,$2,$3,$4)")
            .bind(pedido_id)
            .bind(producto_id)
            .bind(cantidad)
            .bind(precio)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "update productos set stock_actual = stock_actual - $1,
             estado = case when stock_actual - $1 <= 0 then 'agotado' else estado end
             where id = $2",
        )
        .bind(cantidad)
        .bind(producto_id)
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "insert into pagos (pedido_id, metodo, estado, monto) values ($1,$2,'aprobado',$3)",
    )
    .bind(pedido_id)
    .bind(&body.metodo_pago)
    .bind(subtotal)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    if let Some(cliente_id) = body.cliente_id {
        acreditar_puntos(&state.pool, cliente_id, pedido_id, subtotal).await?;
    }
    run_inventory_agent(&state.pool, None).await?;
    write_audit(
        &state.pool,
        Some(body.usuario_sistema_id),
        "venta",
        "pedidos",
        Some(pedido_id),
        None,
        None,
    )
    .await?;
    send_optional_order_email(&state, &body.email_confirmacion, pedido_id, subtotal).await?;

    Ok(HttpResponse::Created().json(json!({"pedido_id": pedido_id, "total": subtotal})))
}

#[get("/api/pedidos")]
async fn listar_pedidos(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    let actor = actor_from_headers(&req)?;
    let rows = if actor.rol.is_staff() {
        sqlx::query("select id, cliente_id, canal, estado, metodo_pago, total, created_at from pedidos order by created_at desc limit 200")
            .fetch_all(&state.pool)
            .await?
    } else if actor.rol == Role::Cliente {
        sqlx::query("select id, cliente_id, canal, estado, metodo_pago, total, created_at from pedidos where cliente_id = $1 order by created_at desc")
            .bind(actor.id)
            .fetch_all(&state.pool)
            .await?
    } else {
        return Err(ApiError::Unauthorized);
    };

    Ok(HttpResponse::Ok().json(
        rows.into_iter()
            .map(|r| {
                json!({
                    "id": r.get::<Uuid, _>("id"),
                    "cliente_id": r.get::<Option<Uuid>, _>("cliente_id"),
                    "canal": r.get::<String, _>("canal"),
                    "estado": r.get::<String, _>("estado"),
                    "metodo_pago": r.get::<Option<String>, _>("metodo_pago"),
                    "total": r.get::<Decimal, _>("total"),
                    "created_at": r.get::<DateTime<Utc>, _>("created_at")
                })
            })
            .collect::<Vec<_>>(),
    ))
}

#[post("/api/devoluciones")]
async fn crear_devolucion(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<CrearDevolucion>,
) -> Result<impl Responder, ApiError> {
    let actor = actor_from_headers(&req)?;
    if !(actor.rol.is_staff() || actor.id == Some(body.cliente_id)) {
        return Err(ApiError::Unauthorized);
    }
    if !matches!(body.tipo.as_str(), "reembolso" | "cambio") {
        return Err(ApiError::BadRequest(
            "tipo debe ser reembolso o cambio".into(),
        ));
    }

    let row =
        sqlx::query("select created_at, total from pedidos where id = $1 and cliente_id = $2")
            .bind(body.pedido_id)
            .bind(body.cliente_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::NotFound)?;
    let created_at: DateTime<Utc> = row.get("created_at");
    if Utc::now() - created_at > Duration::days(7) {
        return Err(ApiError::BadRequest(
            "la devolución está fuera del plazo de 7 días naturales".into(),
        ));
    }

    let monto: Decimal = row.get("total");
    let devolucion_id: Uuid = sqlx::query(
        "insert into devoluciones (pedido_id, detalle_pedido_id, cliente_id, motivo_id, tipo,
                                   producto_sustituto_id, estado, monto_reembolso, procesado_por, fecha_resolucion)
         values ($1,$2,$3,$4,$5,$6,'procesada',$7,$8,now()) returning id",
    )
    .bind(body.pedido_id)
    .bind(body.detalle_pedido_id)
    .bind(body.cliente_id)
    .bind(body.motivo_id)
    .bind(&body.tipo)
    .bind(body.producto_sustituto_id)
    .bind(if body.tipo == "reembolso" { monto } else { Decimal::ZERO })
    .bind(body.procesado_por)
    .fetch_one(&state.pool)
    .await?
    .get("id");

    if let Some(detalle_id) = body.detalle_pedido_id {
        sqlx::query(
            "update productos set stock_actual = stock_actual + dp.cantidad, estado = 'activo'
             from detalle_pedido dp where dp.id = $1 and productos.id = dp.producto_id",
        )
        .bind(detalle_id)
        .execute(&state.pool)
        .await?;
    }

    if let Some(email) = &body.email_confirmacion {
        if state.config.mail_enabled() {
            send_email(
                &state.http,
                &state.config,
                email,
                "Devolución PaperMind",
                "<p>Tu devolución fue registrada correctamente.</p>",
            )
            .await?;
        }
    }

    Ok(
        HttpResponse::Created()
            .json(json!({"devolucion_id": devolucion_id, "estado": "procesada"})),
    )
}

#[get("/api/clientes/{id}/puntos")]
async fn obtener_puntos(
    req: HttpRequest,
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let actor = actor_from_headers(&req)?;
    if !(actor.rol.is_staff() || actor.id == Some(*id)) {
        return Err(ApiError::Unauthorized);
    }
    let row = sqlx::query(
        "select coalesce(sum(case when tipo = 'acumulacion' then puntos else -puntos end), 0)::int as saldo
         from transacciones_puntos where cliente_id = $1",
    )
    .bind(*id)
    .fetch_one(&state.pool)
    .await?;
    Ok(HttpResponse::Ok().json(json!({"cliente_id": *id, "saldo": row.get::<i32, _>("saldo")})))
}

async fn carrito_activo(pool: &sqlx::PgPool, cliente_id: Uuid) -> Result<Uuid, ApiError> {
    if let Some(row) =
        sqlx::query("select id from carritos where cliente_id = $1 and estado = 'activo' limit 1")
            .bind(cliente_id)
            .fetch_optional(pool)
            .await?
    {
        Ok(row.get("id"))
    } else {
        Ok(
            sqlx::query("insert into carritos (cliente_id) values ($1) returning id")
                .bind(cliente_id)
                .fetch_one(pool)
                .await?
                .get("id"),
        )
    }
}

async fn carrito_items(
    pool: &sqlx::PgPool,
    carrito_id: Uuid,
) -> Result<Vec<serde_json::Value>, ApiError> {
    let rows = sqlx::query(
        "select ci.producto_id, p.nombre, ci.cantidad, p.precio_venta, (ci.cantidad * p.precio_venta) as subtotal
         from carrito_items ci join productos p on p.id = ci.producto_id
         where ci.carrito_id = $1 order by p.nombre",
    )
    .bind(carrito_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            json!({
                "producto_id": r.get::<Uuid, _>("producto_id"),
                "nombre": r.get::<String, _>("nombre"),
                "cantidad": r.get::<i32, _>("cantidad"),
                "precio_venta": r.get::<Decimal, _>("precio_venta"),
                "subtotal": r.get::<Decimal, _>("subtotal")
            })
        })
        .collect())
}

fn validar_metodo_pago(metodo: &str) -> Result<(), ApiError> {
    matches!(metodo, "tarjeta" | "transferencia" | "oxxo" | "efectivo")
        .then_some(())
        .ok_or_else(|| ApiError::BadRequest("método de pago no soportado".into()))
}

fn validar_modalidad(modalidad: &str) -> Result<(), ApiError> {
    matches!(modalidad, "domicilio" | "recoleccion" | "mostrador")
        .then_some(())
        .ok_or_else(|| ApiError::BadRequest("modalidad de entrega no soportada".into()))
}

async fn send_optional_order_email(
    state: &AppState,
    email: &Option<String>,
    pedido_id: Uuid,
    total: Decimal,
) -> Result<(), ApiError> {
    if let Some(email) = email {
        if state.config.mail_enabled() {
            let html = format!(
                "<h1>Pedido confirmado</h1><p>Pedido: {pedido_id}</p><p>Total: ${total} MXN</p>"
            );
            send_email(
                &state.http,
                &state.config,
                email,
                "Confirmación de pedido PaperMind",
                &html,
            )
            .await?;
        }
    }
    Ok(())
}
