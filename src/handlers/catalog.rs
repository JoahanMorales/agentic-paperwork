use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, get, patch, post,
    web::{Data, Json, Path, Query},
};
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::require_staff,
    error::ApiError,
    models::catalog::{
        ActualizarProducto, Categoria, CrearCategoria, CrearProducto, Producto, ProductoQuery,
        ProductoSimple,
    },
    services::audit::write_audit,
    state::AppState,
};

const PRODUCTO_SELECT: &str = r#"
select id, nombre, descripcion, categoria_id, precio_venta, precio_costo,
       stock_actual, punto_reorden, proveedor_principal_id, proveedor_alternativo_id,
       codigo_barras_qr, es_temporada, fecha_activacion, fecha_desactivacion, estado, imagen_url
from productos
"#;

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(listar_categorias)
        .service(crear_categoria)
        .service(listar_productos)
        .service(obtener_producto)
        .service(crear_producto)
        .service(actualizar_producto)
        .service(desactivar_producto)
        .service(recomendaciones)
        .service(suscribirse_disponibilidad);
}

#[get("/api/categorias")]
async fn listar_categorias(state: Data<AppState>) -> Result<impl Responder, ApiError> {
    let items = sqlx::query_as::<_, Categoria>(
        "select id, nombre, categoria_padre_id from categorias order by nombre",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(items))
}

#[post("/api/categorias")]
async fn crear_categoria(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<CrearCategoria>,
) -> Result<impl Responder, ApiError> {
    require_staff(&req)?;
    if body.nombre.trim().is_empty() {
        return Err(ApiError::BadRequest("el nombre es obligatorio".into()));
    }

    let item = sqlx::query_as::<_, Categoria>(
        "insert into categorias (nombre, categoria_padre_id) values ($1, $2)
         returning id, nombre, categoria_padre_id",
    )
    .bind(body.nombre.trim())
    .bind(body.categoria_padre_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(HttpResponse::Created().json(item))
}

#[get("/api/productos")]
async fn listar_productos(
    state: Data<AppState>,
    query: Query<ProductoQuery>,
) -> Result<impl Responder, ApiError> {
    let q = query.q.clone().unwrap_or_default();
    let marca = query.marca.clone().unwrap_or_default();
    let min = query.min_precio.unwrap_or(Decimal::ZERO);
    let max = query.max_precio.unwrap_or(Decimal::from(9_999_999));
    let disponible = query.disponible.unwrap_or(false);

    let productos = sqlx::query_as::<_, Producto>(
        r#"
        select id, nombre, descripcion, categoria_id, precio_venta, precio_costo,
               stock_actual, punto_reorden, proveedor_principal_id, proveedor_alternativo_id,
               codigo_barras_qr, es_temporada, fecha_activacion, fecha_desactivacion, estado, imagen_url
        from productos
        where estado <> 'inactivo'
          and ($1 = '' or nombre ilike '%' || $1 || '%' or coalesce(descripcion, '') ilike '%' || $1 || '%')
          and ($2::uuid is null or categoria_id = $2)
          and ($3 = '' or coalesce(descripcion, '') ilike '%' || $3 || '%')
          and precio_venta between $4 and $5
          and ($6 = false or stock_actual > 0)
        order by nombre
        "#,
    )
    .bind(q)
    .bind(query.categoria_id)
    .bind(marca)
    .bind(min)
    .bind(max)
    .bind(disponible)
    .fetch_all(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(productos))
}

#[get("/api/productos/{id}")]
async fn obtener_producto(
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let sql = format!("{PRODUCTO_SELECT} where id = $1");
    let producto = sqlx::query_as::<_, Producto>(&sql)
        .bind(*id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok(HttpResponse::Ok().json(producto))
}

#[post("/api/productos")]
async fn crear_producto(
    req: HttpRequest,
    state: Data<AppState>,
    body: Json<CrearProducto>,
) -> Result<impl Responder, ApiError> {
    let actor = require_staff(&req)?;
    validar_producto(&body)?;

    let estado = if body.stock_actual == 0 {
        "agotado"
    } else {
        "activo"
    };
    let producto = sqlx::query_as::<_, Producto>(
        r#"
        insert into productos
        (nombre, descripcion, categoria_id, precio_venta, precio_costo, stock_actual, punto_reorden,
         proveedor_principal_id, proveedor_alternativo_id, codigo_barras_qr, es_temporada,
         fecha_activacion, fecha_desactivacion, estado, imagen_url)
        values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
        returning id, nombre, descripcion, categoria_id, precio_venta, precio_costo,
                  stock_actual, punto_reorden, proveedor_principal_id, proveedor_alternativo_id,
                  codigo_barras_qr, es_temporada, fecha_activacion, fecha_desactivacion, estado, imagen_url
        "#,
    )
    .bind(body.nombre.trim())
    .bind(&body.descripcion)
    .bind(body.categoria_id)
    .bind(body.precio_venta)
    .bind(body.precio_costo)
    .bind(body.stock_actual)
    .bind(body.punto_reorden.unwrap_or(0))
    .bind(body.proveedor_principal_id)
    .bind(body.proveedor_alternativo_id)
    .bind(body.codigo_barras_qr.trim())
    .bind(body.es_temporada.unwrap_or(false))
    .bind(body.fecha_activacion)
    .bind(body.fecha_desactivacion)
    .bind(estado)
    .bind(&body.imagen_url)
    .fetch_one(&state.pool)
    .await?;

    write_audit(
        &state.pool,
        actor.id,
        "crear",
        "productos",
        Some(producto.id),
        None,
        None,
    )
    .await?;
    Ok(HttpResponse::Created().json(producto))
}

#[patch("/api/productos/{id}")]
async fn actualizar_producto(
    req: HttpRequest,
    state: Data<AppState>,
    id: Path<Uuid>,
    body: Json<ActualizarProducto>,
) -> Result<impl Responder, ApiError> {
    let actor = require_staff(&req)?;
    let actual = get_producto_simple(&state.pool, *id).await?;
    let stock = body.stock_actual.unwrap_or(actual.stock_actual);
    if stock < 0 {
        return Err(ApiError::BadRequest(
            "el stock no puede ser negativo".into(),
        ));
    }
    let estado = body.estado.clone().unwrap_or_else(|| {
        if stock == 0 {
            "agotado".into()
        } else {
            "activo".into()
        }
    });

    let producto = sqlx::query_as::<_, Producto>(
        r#"
        update productos set
          nombre = coalesce($2, nombre), descripcion = coalesce($3, descripcion),
          categoria_id = coalesce($4, categoria_id), precio_venta = coalesce($5, precio_venta),
          precio_costo = coalesce($6, precio_costo), stock_actual = $7,
          punto_reorden = coalesce($8, punto_reorden),
          proveedor_principal_id = coalesce($9, proveedor_principal_id),
          proveedor_alternativo_id = coalesce($10, proveedor_alternativo_id),
          estado = $11, imagen_url = coalesce($12, imagen_url)
        where id = $1
        returning id, nombre, descripcion, categoria_id, precio_venta, precio_costo,
                  stock_actual, punto_reorden, proveedor_principal_id, proveedor_alternativo_id,
                  codigo_barras_qr, es_temporada, fecha_activacion, fecha_desactivacion, estado, imagen_url
        "#,
    )
    .bind(*id)
    .bind(&body.nombre)
    .bind(&body.descripcion)
    .bind(body.categoria_id)
    .bind(body.precio_venta)
    .bind(body.precio_costo)
    .bind(stock)
    .bind(body.punto_reorden)
    .bind(body.proveedor_principal_id)
    .bind(body.proveedor_alternativo_id)
    .bind(&estado)
    .bind(&body.imagen_url)
    .fetch_one(&state.pool)
    .await?;

    write_audit(
        &state.pool,
        actor.id,
        "actualizar",
        "productos",
        Some(*id),
        None,
        None,
    )
    .await?;
    if actual.precio_venta != producto.precio_venta {
        write_audit(
            &state.pool,
            actor.id,
            "modificar_precio",
            "productos",
            Some(*id),
            None,
            None,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().json(producto))
}

#[delete("/api/productos/{id}")]
async fn desactivar_producto(
    req: HttpRequest,
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let actor = require_staff(&req)?;
    sqlx::query("update productos set estado = 'inactivo' where id = $1")
        .bind(*id)
        .execute(&state.pool)
        .await?;
    write_audit(
        &state.pool,
        actor.id,
        "eliminar",
        "productos",
        Some(*id),
        None,
        None,
    )
    .await?;
    Ok(HttpResponse::Ok().json(json!({"mensaje": "producto desactivado"})))
}

#[get("/api/productos/{id}/recomendaciones")]
async fn recomendaciones(
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    get_producto_simple(&state.pool, *id).await?;
    let rows = sqlx::query(
        "select p.id, p.nombre, p.precio_venta, p.stock_actual
         from productos p
         where p.estado = 'activo' and p.stock_actual > 0 and p.id <> $1
           and p.categoria_id = (select categoria_id from productos where id = $1)
         order by p.stock_actual desc limit 5",
    )
    .bind(*id)
    .fetch_all(&state.pool)
    .await?;

    Ok(HttpResponse::Ok().json(
        rows.into_iter()
            .map(|r| {
                let precio: Decimal = r.get("precio_venta");
                json!({
                    "id": r.get::<Uuid, _>("id"),
                    "nombre": r.get::<String, _>("nombre"),
                    "precio_venta": precio,
                    "stock_actual": r.get::<i32, _>("stock_actual")
                })
            })
            .collect::<Vec<_>>(),
    ))
}

#[post("/api/productos/{id}/suscripciones-disponibilidad")]
async fn suscribirse_disponibilidad(
    req: HttpRequest,
    state: Data<AppState>,
    id: Path<Uuid>,
) -> Result<impl Responder, ApiError> {
    let actor = crate::auth::require_cliente(&req)?;
    sqlx::query(
        "insert into suscripciones_disponibilidad (cliente_id, producto_id) values ($1,$2)
         on conflict (cliente_id, producto_id) do nothing",
    )
    .bind(actor.id.unwrap())
    .bind(*id)
    .execute(&state.pool)
    .await?;
    Ok(HttpResponse::Ok().json(
        json!({"mensaje": "te avisaremos por correo cuando el producto vuelva a tener stock"}),
    ))
}

fn validar_producto(body: &CrearProducto) -> Result<(), ApiError> {
    if body.nombre.trim().is_empty()
        || body.codigo_barras_qr.trim().is_empty()
        || body.precio_venta < Decimal::ZERO
        || body.precio_costo < Decimal::ZERO
        || body.stock_actual < 0
    {
        return Err(ApiError::BadRequest(
            "producto inválido: nombre, categoría, precio, costo, stock, proveedor y código QR/barras son obligatorios".into(),
        ));
    }
    Ok(())
}

pub async fn get_producto_simple(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> Result<ProductoSimple, ApiError> {
    sqlx::query_as::<_, ProductoSimple>(
        "select id, nombre, precio_venta, precio_costo, stock_actual, punto_reorden, proveedor_principal_id
         from productos where id = $1 and estado <> 'inactivo'",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}
