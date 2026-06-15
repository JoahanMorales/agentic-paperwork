use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize, FromRow)]
pub struct Proveedor {
    pub id: Uuid,
    pub nombre: String,
    pub contacto_nombre: Option<String>,
    pub correo: Option<String>,
    pub telefono: Option<String>,
    pub canal_digital: bool,
    pub tiene_orden_previa_exitosa: bool,
    pub calificacion_desempeno: Decimal,
    pub prioridad: i32,
    pub estado: String,
}

#[derive(Deserialize)]
pub struct CrearProveedor {
    pub nombre: String,
    pub contacto_nombre: Option<String>,
    pub correo: Option<String>,
    pub telefono: Option<String>,
    pub canal_digital: Option<bool>,
    pub prioridad: Option<i32>,
}
