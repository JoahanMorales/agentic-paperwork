use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, put,
    web::{Data, Json},
};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{require_admin, require_staff},
    error::ApiError,
    services::{
        audit::write_audit, inventory::run_inventory_agent, mail::send_email,
        openrouter::chat_completion,
    },
    state::AppState,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(agente_conversacional)
        .service(ejecutar_agente_inventario)
        .service(ejecutar_prediccion)
        .service(configurar_agente)
        .service(generar_promocion)
        .service(dashboard)
        .service(reporte_ventas);
}

#[derive(Deserialize)]
struct ChatRequest {
    cliente_id: Option<Uuid>,
    mensaje: String,
}

#[post("/api/agentes/conversacional/mensaje")]
async fn agente_conversacional(
    state: Data<AppState>,
    body: Json<ChatRequest>,
) -> Result<impl Responder, ApiError> {
    let catalogo = catalog_context(&state).await?;
    let system = "Eres el agente conversacional de PaperMind, una papelería mexicana. Responde en español mexicano, claro y amable. Solo puedes hablar de catálogo, disponibilidad, precios, pedidos y devoluciones. No inventes precios ni descuentos. Si el usuario pide algo fuera de dominio, indica que debe escalarse a humano.";
    let prompt = format!(
        "Catálogo disponible resumido:\n{catalogo}\n\nMensaje del cliente: {}\n\nDevuelve una respuesta útil y breve. Si requiere humano, inicia con [ESCALAR].",
        body.mensaje
    );
    let respuesta = chat_completion(&state.http, &state.config, system, &prompt).await?;
    let escalado = respuesta.trim_start().starts_with("[ESCALAR]");
    let intencion = detectar_intencion(&body.mensaje);

    let interaccion_id: Uuid = sqlx::query(
        "insert into interacciones_agente (cliente_id, canal, consulta_resumen, intencion_detectada, escalado, sentimiento, resuelta, fecha_fin)
         values ($1,'web',$2,$3,$4,'neutro',$5,now()) returning id",
    )
    .bind(body.cliente_id)
    .bind(body.mensaje.chars().take(240).collect::<String>())
    .bind(intencion)
    .bind(escalado)
    .bind(!escalado)
    .fetch_one(&state.pool)
    .await?
    .get("id");

    Ok(HttpResponse::Ok().json(json!({
        "interaccion_id": interaccion_id,
        "intencion": intencion,
        "escalado": escalado,
        "respuesta": respuesta.replace("[ESCALAR]", "").trim()
    })))
}

#[post("/api/agentes/inventario/ejecutar")]
async fn ejecutar_agente_inventario(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    Ok(HttpResponse::Ok().json(run_inventory_agent(&state.pool, None).await?))
}

#[post("/api/agentes/prediccion/ejecutar")]
async fn ejecutar_prediccion(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    let rows = sqlx::query(
        "select p.id, p.nombre, p.stock_actual,
                coalesce(sum(case when pe.created_at >= now() - interval '90 days' then dp.cantidad else 0 end), 0)::int as unidades_90
         from productos p
         left join detalle_pedido dp on dp.producto_id = p.id
         left join pedidos pe on pe.id = dp.pedido_id
         where p.estado <> 'inactivo'
         group by p.id, p.nombre, p.stock_actual
         order by p.nombre",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut resumen = String::new();
    let mut generadas = 0;
    for row in rows {
        let producto_id: Uuid = row.get("id");
        let nombre: String = row.get("nombre");
        let stock: i32 = row.get("stock_actual");
        let unidades_90: i32 = row.get("unidades_90");
        resumen.push_str(&format!(
            "- {nombre}: stock {stock}, ventas 90d {unidades_90}\n"
        ));

        let promedio_diario = (unidades_90 as f64 / 90.0).max(0.05);
        for periodo in [30, 60, 90] {
            let demanda =
                (promedio_diario * periodo as f64 * factor_estacionalidad()).ceil() as i32;
            let riesgo = if stock < demanda {
                "rojo"
            } else if stock < demanda * 2 {
                "amarillo"
            } else {
                "verde"
            };
            let confianza = if unidades_90 >= 30 {
                "alto"
            } else if unidades_90 >= 10 {
                "medio"
            } else {
                "bajo"
            };
            sqlx::query(
                "insert into predicciones_demanda (producto_id, periodo_dias, demanda_estimada, nivel_confianza, nivel_riesgo)
                 values ($1,$2,$3,$4,$5)",
            )
            .bind(producto_id)
            .bind(periodo)
            .bind(demanda)
            .bind(confianza)
            .bind(riesgo)
            .execute(&state.pool)
            .await?;
            generadas += 1;
        }
    }

    let insight = chat_completion(
        &state.http,
        &state.config,
        "Eres un agente analítico de inventario para una papelería mexicana. Da recomendaciones accionables y breves.",
        &format!("Analiza estos datos y sugiere compras, riesgos de desabasto y liquidaciones:\n{resumen}"),
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "predicciones_generadas": generadas,
        "insight_ia": insight
    })))
}

#[derive(Deserialize)]
struct ConfigAgenteRequest {
    agente: String,
    parametro: String,
    valor: String,
}

#[put("/api/agentes/configuracion")]
async fn configurar_agente(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<ConfigAgenteRequest>,
) -> Result<impl Responder, ApiError> {
    let actor = require_admin(&req)?;
    if !matches!(
        body.agente.as_str(),
        "inventario" | "conversacional" | "prediccion" | "fidelizacion" | "recomendacion"
    ) {
        return Err(ApiError::BadRequest("agente no válido".into()));
    }

    sqlx::query(
        "insert into configuracion_agentes (agente, parametro, valor, actualizado_por)
         values ($1,$2,$3,$4)
         on conflict (agente, parametro) do update set valor = excluded.valor, actualizado_por = excluded.actualizado_por, fecha_actualizacion = now()",
    )
    .bind(&body.agente)
    .bind(&body.parametro)
    .bind(&body.valor)
    .bind(actor.id)
    .execute(&state.pool)
    .await?;
    write_audit(
        &state.pool,
        actor.id,
        "configurar_agente",
        "configuracion_agentes",
        None,
        None,
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(json!({"mensaje": "configuración actualizada"})))
}

#[derive(Deserialize)]
struct PromoRequest {
    cliente_id: Uuid,
    email: String,
}

#[post("/api/agentes/fidelizacion/promocion")]
async fn generar_promocion(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<PromoRequest>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    let cliente =
        sqlx::query("select nombre, tipo_perfil, es_cliente_frecuente from clientes where id = $1")
            .bind(body.cliente_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::NotFound)?;

    let compras = sqlx::query(
        "select p.nombre, sum(dp.cantidad)::int as cantidad
         from pedidos pe
         join detalle_pedido dp on dp.pedido_id = pe.id
         join productos p on p.id = dp.producto_id
         where pe.cliente_id = $1
         group by p.nombre
         order by cantidad desc limit 5",
    )
    .bind(body.cliente_id)
    .fetch_all(&state.pool)
    .await?;

    let historial = compras
        .into_iter()
        .map(|r| {
            format!(
                "{} ({})",
                r.get::<String, _>("nombre"),
                r.get::<i32, _>("cantidad")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let prompt = format!(
        "Cliente: {}, perfil {}, frecuente: {}. Historial: {}. Genera un correo promocional breve con asunto y cuerpo HTML. No prometas descuentos no configurados; ofrece recomendación personalizada.",
        cliente.get::<String, _>("nombre"),
        cliente.get::<String, _>("tipo_perfil"),
        cliente.get::<bool, _>("es_cliente_frecuente"),
        historial
    );
    let contenido = chat_completion(
        &state.http,
        &state.config,
        "Eres agente de fidelización de PaperMind.",
        &prompt,
    )
    .await?;

    if state.config.mail_enabled() {
        send_email(
            &state.http,
            &state.config,
            &body.email,
            "Promoción personalizada PaperMind",
            &contenido,
        )
        .await?;
        sqlx::query(
            "insert into comunicaciones_enviadas (cliente_id, canal, contenido) values ($1,'correo',$2)",
        )
        .bind(body.cliente_id)
        .bind(&contenido)
        .execute(&state.pool)
        .await?;
    }

    Ok(HttpResponse::Ok().json(json!({
        "cliente_id": body.cliente_id,
        "correo_enviado": state.config.mail_enabled(),
        "contenido": contenido
    })))
}

#[get("/api/dashboard")]
async fn dashboard(req: HttpRequest, state: Data<AppState>) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    let ventas = sqlx::query(
        "select coalesce(sum(total),0) as total from pedidos where created_at::date = current_date",
    )
    .fetch_one(&state.pool)
    .await?;
    let pendientes = sqlx::query("select count(*)::int as total from pedidos where estado in ('pendiente_pago','en_proceso')")
        .fetch_one(&state.pool)
        .await?;
    let stock = sqlx::query("select count(*)::int as total from productos where stock_actual <= punto_reorden and estado <> 'inactivo'")
        .fetch_one(&state.pool)
        .await?;
    let alertas =
        sqlx::query("select count(*)::int as total from alertas where estado = 'pendiente'")
            .fetch_one(&state.pool)
            .await?;

    Ok(HttpResponse::Ok().json(json!({
        "ventas_hoy": ventas.get::<rust_decimal::Decimal, _>("total"),
        "pedidos_pendientes": pendientes.get::<i32, _>("total"),
        "productos_alerta_stock": stock.get::<i32, _>("total"),
        "alertas_pendientes": alertas.get::<i32, _>("total"),
        "openrouter_configurado": state.config.openrouter_enabled(),
        "correo_configurado": state.config.mail_enabled()
    })))
}

#[get("/api/reportes/ventas")]
async fn reporte_ventas(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    let row = sqlx::query(
        "select coalesce(sum(total),0) as ventas, count(*)::int as pedidos
         from pedidos where created_at::date = current_date and estado in ('pagado','completado')",
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "periodo": "hoy",
        "ventas_totales": row.get::<rust_decimal::Decimal, _>("ventas"),
        "pedidos": row.get::<i32, _>("pedidos")
    })))
}

async fn catalog_context(state: &AppState) -> Result<String, ApiError> {
    let rows = sqlx::query(
        "select nombre, precio_venta, stock_actual, estado
         from productos where estado <> 'inactivo'
         order by stock_actual desc limit 25",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            format!(
                "{} | ${} | stock {} | {}",
                r.get::<String, _>("nombre"),
                r.get::<rust_decimal::Decimal, _>("precio_venta"),
                r.get::<i32, _>("stock_actual"),
                r.get::<String, _>("estado")
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

fn detectar_intencion(mensaje: &str) -> &'static str {
    let lower = mensaje.to_lowercase();
    if lower.contains("precio") || lower.contains("cuesta") {
        "precio"
    } else if lower.contains("stock") || lower.contains("disponible") || lower.contains("hay") {
        "disponibilidad"
    } else if lower.contains("pedido") || lower.contains("orden") {
        "estado_pedido"
    } else if lower.contains("devol") || lower.contains("cambio") {
        "devolucion"
    } else {
        "general"
    }
}

fn factor_estacionalidad() -> f64 {
    match Utc::now().month() {
        7 | 8 | 9 => 1.35,
        12 | 1 => 1.15,
        _ => 1.0,
    }
}
