use actix_web::{
    HttpRequest, HttpResponse, Responder, get, post,
    web::{Data, Json},
};

use crate::{
    auth::require_staff,
    error::ApiError,
    models::provider::{CrearProveedor, Proveedor},
    state::AppState,
};

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(listar_proveedores).service(crear_proveedor);
}

#[get("/api/proveedores")]
async fn listar_proveedores(
    req: HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    let proveedores = sqlx::query_as::<_, Proveedor>(
        "select id, nombre, contacto_nombre, correo, telefono, canal_digital, tiene_orden_previa_exitosa,
                calificacion_desempeno, prioridad, estado
         from proveedores order by prioridad desc, nombre",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(proveedores))
}

#[post("/api/proveedores")]
async fn crear_proveedor(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<CrearProveedor>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    if body.nombre.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "el nombre del proveedor es obligatorio".into(),
        ));
    }

    let proveedor = sqlx::query_as::<_, Proveedor>(
        "insert into proveedores (nombre, contacto_nombre, correo, telefono, canal_digital, prioridad)
         values ($1,$2,$3,$4,$5,$6)
         returning id, nombre, contacto_nombre, correo, telefono, canal_digital, tiene_orden_previa_exitosa,
                   calificacion_desempeno, prioridad, estado",
    )
    .bind(body.nombre.trim())
    .bind(&body.contacto_nombre)
    .bind(&body.correo)
    .bind(&body.telefono)
    .bind(body.canal_digital.unwrap_or(false))
    .bind(body.prioridad.unwrap_or(0))
    .fetch_one(&state.pool)
    .await?;

    Ok(HttpResponse::Created().json(proveedor))
}
