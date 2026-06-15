use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post, put,
    web::{Data, Json},
};
use chrono::{Datelike, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
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
        .service(enviar_correo_prueba)
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
    let intencion = detectar_intencion(&body.mensaje);
    let sentimiento = detectar_sentimiento(&body.mensaje);
    let (catalogo, productos_sugeridos) = relevant_products_context(&state, &body.mensaje).await?;
    let cliente_context = customer_context(&state, body.cliente_id).await?;
    let pedido_context = order_context_from_message(&state, &body.mensaje).await?;

    let system = r#"Eres PaperMind IA, el agente conversacional oficial de una papelería mexicana.
Reglas obligatorias:
- Responde siempre en español mexicano, amable, breve y útil.
- Usa SOLO la información del catálogo y contexto proporcionados. No inventes precios, stock, descuentos ni políticas.
- Puedes ayudar con catálogo, disponibilidad, precios, recomendaciones, pedidos, puntos y devoluciones.
- Si el cliente pide algo fuera del dominio de PaperMind, marca escalado=true.
- Si el cliente está molesto o hay una devolución compleja, marca escalado=true.
- Si un producto no tiene stock, ofrece alternativas del catálogo o suscripción de disponibilidad.
- Devuelve exclusivamente JSON válido, sin markdown, con esta forma:
{
  "respuesta": "texto para el cliente",
  "escalado": false,
  "motivo_escalado": null,
  "acciones_sugeridas": ["accion corta"],
  "productos_mencionados": ["nombre exacto del producto"]
}
"#;

    let prompt = format!(
        "Intención detectada: {intencion}\nSentimiento detectado: {sentimiento}\n\nContexto del cliente:\n{cliente_context}\n\nContexto de pedido detectado:\n{pedido_context}\n\nProductos relevantes del catálogo:\n{catalogo}\n\nMensaje del cliente:\n{}",
        body.mensaje
    );
    let raw_response = chat_completion(&state.http, &state.config, system, &prompt).await?;
    let parsed = parse_agent_response(&raw_response);
    let escalado = parsed
        .get("escalado")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| sentimiento == "frustracion" || intencion == "fuera_dominio");
    let respuesta = parsed
        .get("respuesta")
        .and_then(Value::as_str)
        .unwrap_or(raw_response.trim());

    let interaccion_id: Uuid = sqlx::query(
        "insert into interacciones_agente (cliente_id, canal, consulta_resumen, intencion_detectada, escalado, sentimiento, resuelta, fecha_fin)
         values ($1,'web',$2,$3,$4,$5,$6,now()) returning id",
    )
    .bind(body.cliente_id)
    .bind(body.mensaje.chars().take(240).collect::<String>())
    .bind(intencion)
    .bind(escalado)
    .bind(sentimiento)
    .bind(!escalado)
    .fetch_one(&state.pool)
    .await?
    .get("id");

    Ok(HttpResponse::Ok().json(json!({
        "interaccion_id": interaccion_id,
        "intencion": intencion,
        "sentimiento": sentimiento,
        "escalado": escalado,
        "motivo_escalado": parsed.get("motivo_escalado").cloned().unwrap_or(Value::Null),
        "respuesta": respuesta,
        "acciones_sugeridas": parsed.get("acciones_sugeridas").cloned().unwrap_or_else(|| json!([])),
        "productos_mencionados": parsed.get("productos_mencionados").cloned().unwrap_or_else(|| json!([])),
        "productos_sugeridos": productos_sugeridos
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

#[derive(Deserialize)]
struct TestEmailRequest {
    email: String,
}

#[post("/api/mail/test")]
async fn enviar_correo_prueba(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<TestEmailRequest>,
) -> Result<impl Responder, ApiError> {
    require_admin(&req)?;
    let resend_response = send_email(
        &state.http,
        &state.config,
        &body.email,
        "Prueba de correo PaperMind",
        "<h1>PaperMind</h1><p>Este es un correo de prueba enviado desde el backend en Render.</p>",
    )
    .await?;

    Ok(HttpResponse::Ok().json(json!({
        "correo_enviado": true,
        "resend": resend_response
    })))
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

    let mut resend_response = Value::Null;
    if state.config.mail_enabled() {
        resend_response = send_email(
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
        "resend": resend_response,
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

async fn relevant_products_context(
    state: &AppState,
    message: &str,
) -> Result<(String, Vec<Value>), ApiError> {
    let rows = sqlx::query(
        "select p.id, p.nombre, p.descripcion, p.precio_venta, p.stock_actual, p.estado,
                coalesce(c.nombre, 'Sin categoría') as categoria
         from productos p
         left join categorias c on c.id = p.categoria_id
         where p.estado <> 'inactivo'
         order by p.stock_actual desc, p.nombre
         limit 150",
    )
    .fetch_all(&state.pool)
    .await?;

    let terms = search_terms(message);
    let mut scored = rows
        .into_iter()
        .map(|row| {
            let name: String = row.get("nombre");
            let description: Option<String> = row.get("descripcion");
            let category: String = row.get("categoria");
            let searchable = format!(
                "{} {} {}",
                name.to_lowercase(),
                description.clone().unwrap_or_default().to_lowercase(),
                category.to_lowercase()
            );
            let score = terms
                .iter()
                .filter(|term| searchable.contains(term.as_str()))
                .count();
            (score, row, name, description, category)
        })
        .collect::<Vec<_>>();

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let selected = scored
        .into_iter()
        .filter(|(score, _, _, _, _)| terms.is_empty() || *score > 0)
        .take(12)
        .collect::<Vec<_>>();

    let products = selected
        .into_iter()
        .map(|(_, row, name, description, category)| {
            let id: Uuid = row.get("id");
            let price: rust_decimal::Decimal = row.get("precio_venta");
            let stock: i32 = row.get("stock_actual");
            let status: String = row.get("estado");
            json!({
                "id": id,
                "nombre": name,
                "categoria": category,
                "descripcion": description,
                "precio_venta": price,
                "stock_actual": stock,
                "estado": status
            })
        })
        .collect::<Vec<_>>();

    let context = if products.is_empty() {
        "No se encontraron productos relacionados con el mensaje.".to_string()
    } else {
        products
            .iter()
            .map(|product| {
                format!(
                    "- {} | categoría: {} | precio: ${} MXN | stock: {} | estado: {} | id: {}",
                    product["nombre"].as_str().unwrap_or(""),
                    product["categoria"].as_str().unwrap_or(""),
                    product["precio_venta"],
                    product["stock_actual"],
                    product["estado"].as_str().unwrap_or(""),
                    product["id"]
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    Ok((context, products))
}

async fn customer_context(state: &AppState, cliente_id: Option<Uuid>) -> Result<String, ApiError> {
    let Some(cliente_id) = cliente_id else {
        return Ok("Cliente no identificado.".to_string());
    };

    let Some(cliente) = sqlx::query(
        "select nombre, apellido, tipo_perfil, es_cliente_frecuente from clientes where id = $1",
    )
    .bind(cliente_id)
    .fetch_optional(&state.pool)
    .await?
    else {
        return Ok("Cliente enviado no existe en la base.".to_string());
    };

    let puntos = sqlx::query(
        "select coalesce(sum(case when tipo = 'acumulacion' then puntos else -puntos end), 0)::int as saldo
         from transacciones_puntos where cliente_id = $1",
    )
    .bind(cliente_id)
    .fetch_one(&state.pool)
    .await?
    .get::<i32, _>("saldo");

    let last_order = sqlx::query(
        "select id, estado, total, created_at from pedidos where cliente_id = $1 order by created_at desc limit 1",
    )
    .bind(cliente_id)
    .fetch_optional(&state.pool)
    .await?;

    let order_text = if let Some(order) = last_order {
        format!(
            "Último pedido: {} | estado: {} | total: ${} | fecha: {}",
            order.get::<Uuid, _>("id"),
            order.get::<String, _>("estado"),
            order.get::<rust_decimal::Decimal, _>("total"),
            order.get::<chrono::DateTime<Utc>, _>("created_at")
        )
    } else {
        "Sin pedidos registrados.".to_string()
    };

    Ok(format!(
        "Cliente: {} {} | perfil: {} | frecuente: {} | puntos: {}\n{}",
        cliente.get::<String, _>("nombre"),
        cliente.get::<String, _>("apellido"),
        cliente.get::<String, _>("tipo_perfil"),
        cliente.get::<bool, _>("es_cliente_frecuente"),
        puntos,
        order_text
    ))
}

async fn order_context_from_message(state: &AppState, message: &str) -> Result<String, ApiError> {
    let Some(order_id) = extract_uuid(message) else {
        return Ok("No se detectó UUID de pedido en el mensaje.".to_string());
    };

    let Some(order) = sqlx::query(
        "select id, estado, metodo_pago, modalidad_entrega, total, created_at from pedidos where id = $1",
    )
    .bind(order_id)
    .fetch_optional(&state.pool)
    .await?
    else {
        return Ok(format!("Se detectó el UUID {order_id}, pero no existe como pedido."));
    };

    Ok(format!(
        "Pedido detectado: {} | estado: {} | pago: {} | entrega: {} | total: ${} | fecha: {}",
        order.get::<Uuid, _>("id"),
        order.get::<String, _>("estado"),
        order
            .get::<Option<String>, _>("metodo_pago")
            .unwrap_or_else(|| "no registrado".to_string()),
        order
            .get::<Option<String>, _>("modalidad_entrega")
            .unwrap_or_else(|| "no registrada".to_string()),
        order.get::<rust_decimal::Decimal, _>("total"),
        order.get::<chrono::DateTime<Utc>, _>("created_at")
    ))
}

fn search_terms(message: &str) -> Vec<String> {
    let stopwords = [
        "que",
        "con",
        "para",
        "tienen",
        "tengo",
        "quiero",
        "cuanto",
        "cuesta",
        "precio",
        "disponible",
        "hay",
        "los",
        "las",
        "una",
        "uno",
        "del",
        "por",
        "favor",
        "dame",
        "sobre",
        "producto",
        "productos",
    ];

    message
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| word.len() > 2 && !stopwords.contains(word))
        .map(ToString::to_string)
        .collect()
}

fn extract_uuid(message: &str) -> Option<Uuid> {
    message
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '.')
        .find_map(|piece| Uuid::parse_str(piece.trim()).ok())
}

fn parse_agent_response(raw: &str) -> Value {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(cleaned).unwrap_or_else(|_| {
        json!({
            "respuesta": raw.trim(),
            "escalado": false,
            "motivo_escalado": null,
            "acciones_sugeridas": [],
            "productos_mencionados": []
        })
    })
}

fn detectar_intencion(mensaje: &str) -> &'static str {
    let lower = mensaje.to_lowercase();
    if lower.contains("precio") || lower.contains("cuesta") || lower.contains("cuestan") {
        "precio"
    } else if lower.contains("stock") || lower.contains("disponible") || lower.contains("hay") {
        "disponibilidad"
    } else if lower.contains("pedido") || lower.contains("orden") || lower.contains("rastreo") {
        "estado_pedido"
    } else if lower.contains("devol") || lower.contains("cambio") || lower.contains("reembolso") {
        "devolucion"
    } else if lower.contains("recomienda") || lower.contains("sugiere") || lower.contains("similar")
    {
        "recomendacion"
    } else if lower.contains("tarea") || lower.contains("examen") || lower.contains("resuelve") {
        "fuera_dominio"
    } else {
        "general"
    }
}

fn detectar_sentimiento(mensaje: &str) -> &'static str {
    let lower = mensaje.to_lowercase();
    if [
        "molesto",
        "enojado",
        "pésimo",
        "pesimo",
        "mal servicio",
        "no sirve",
        "queja",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        "frustracion"
    } else if ["gracias", "excelente", "perfecto", "genial", "bien"]
        .iter()
        .any(|word| lower.contains(word))
    {
        "positivo"
    } else {
        "neutro"
    }
}

fn factor_estacionalidad() -> f64 {
    match Utc::now().month() {
        7 | 8 | 9 => 1.35,
        12 | 1 => 1.15,
        _ => 1.0,
    }
}
