use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Categoria {
    pub id: Uuid,
    pub nombre: String,
    pub categoria_padre_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CrearCategoria {
    pub nombre: String,
    pub categoria_padre_id: Option<Uuid>,
}

#[derive(Serialize, FromRow)]
pub struct Producto {
    pub id: Uuid,
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: Option<Uuid>,
    pub precio_venta: Decimal,
    pub precio_costo: Decimal,
    pub stock_actual: i32,
    pub punto_reorden: i32,
    pub proveedor_principal_id: Option<Uuid>,
    pub proveedor_alternativo_id: Option<Uuid>,
    pub codigo_barras_qr: String,
    pub es_temporada: bool,
    pub fecha_activacion: Option<NaiveDate>,
    pub fecha_desactivacion: Option<NaiveDate>,
    pub estado: String,
    pub imagen_url: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct ProductoSimple {
    pub id: Uuid,
    pub nombre: String,
    pub precio_venta: Decimal,
    pub precio_costo: Decimal,
    pub stock_actual: i32,
    pub punto_reorden: i32,
    pub proveedor_principal_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct ProductoQuery {
    pub q: Option<String>,
    pub categoria_id: Option<Uuid>,
    pub marca: Option<String>,
    pub min_precio: Option<Decimal>,
    pub max_precio: Option<Decimal>,
    pub disponible: Option<bool>,
}

#[derive(Deserialize)]
pub struct CrearProducto {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub categoria_id: Uuid,
    pub precio_venta: Decimal,
    pub precio_costo: Decimal,
    pub stock_actual: i32,
    pub punto_reorden: Option<i32>,
    pub proveedor_principal_id: Uuid,
    pub proveedor_alternativo_id: Option<Uuid>,
    pub codigo_barras_qr: String,
    pub es_temporada: Option<bool>,
    pub fecha_activacion: Option<NaiveDate>,
    pub fecha_desactivacion: Option<NaiveDate>,
    pub imagen_url: Option<String>,
}

#[derive(Deserialize)]
pub struct ActualizarProducto {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub categoria_id: Option<Uuid>,
    pub precio_venta: Option<Decimal>,
    pub precio_costo: Option<Decimal>,
    pub stock_actual: Option<i32>,
    pub punto_reorden: Option<i32>,
    pub proveedor_principal_id: Option<Uuid>,
    pub proveedor_alternativo_id: Option<Uuid>,
    pub estado: Option<String>,
    pub imagen_url: Option<String>,
}
