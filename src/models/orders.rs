use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AgregarCarritoItem {
    pub producto_id: Uuid,
    pub cantidad: i32,
}

#[derive(Deserialize)]
pub struct ConfirmarPedido {
    pub metodo_pago: String,
    pub modalidad_entrega: String,
    pub direccion_entrega: Option<String>,
    pub puntos_utilizados: Option<i32>,
    pub email_confirmacion: Option<String>,
}

#[derive(Deserialize)]
pub struct CrearVentaFisica {
    pub usuario_sistema_id: Uuid,
    pub cliente_id: Option<Uuid>,
    pub metodo_pago: String,
    pub items: Vec<VentaItem>,
    pub email_confirmacion: Option<String>,
}

#[derive(Deserialize)]
pub struct VentaItem {
    pub producto_id: Uuid,
    pub cantidad: i32,
}

#[derive(Deserialize)]
pub struct CrearDevolucion {
    pub pedido_id: Uuid,
    pub detalle_pedido_id: Option<Uuid>,
    pub cliente_id: Uuid,
    pub motivo_id: Uuid,
    pub tipo: String,
    pub producto_sustituto_id: Option<Uuid>,
    pub procesado_por: Option<Uuid>,
    pub email_confirmacion: Option<String>,
}
