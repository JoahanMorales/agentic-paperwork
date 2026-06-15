# PaperMind Backend

Backend REST hecho únicamente con Rust para PaperMind.

## Stack

- `actix-web`: API HTTP.
- `sqlx`: conexión PostgreSQL/Supabase.
- `reqwest`: consumo de OpenRouter y Resend.
- `serde`: JSON.
- `rust_decimal`: importes monetarios.

## Estructura

```text
src/
  main.rs              # arranque del servidor y registro de rutas
  config.rs            # variables de entorno
  auth.rs              # extracción simple de actor/rol
  error.rs             # errores HTTP de la API
  state.rs             # estado compartido
  handlers/            # controladores HTTP por dominio
    agents.rs
    catalog.rs
    orders.rs
    providers.rs
  models/              # DTOs y structs de consulta
  services/            # lógica reutilizable e integraciones
    audit.rs
    inventory.rs
    loyalty.rs
    mail.rs            # Resend
    openrouter.rs      # OpenRouter
```

## Configuración

1. Ejecuta en Supabase el SQL del esquema de PaperMind.
2. Copia `.env.example` a `.env`.
3. Configura:
   - `DATABASE_URL`
   - `SUPABASE_URL`
   - `SUPABASE_ANON_KEY`
   - `OPENROUTER_API_KEY`
   - `RESEND_API_KEY`
   - `MAIL_FROM`
4. Ejecuta:

```bash
cargo run
```

## Agentes IA

Los agentes ya llaman a OpenRouter cuando están configuradas estas variables:

```env
OPENROUTER_API_KEY=sk-or-v1-...
OPENROUTER_MODEL=openai/gpt-4o-mini
```

Endpoints principales:

- `POST /api/agentes/conversacional/mensaje`
- `POST /api/agentes/prediccion/ejecutar`
- `POST /api/agentes/fidelizacion/promocion`
- `POST /api/agentes/inventario/ejecutar`

Si `OPENROUTER_API_KEY` no existe, esos endpoints devuelven error de servicio no configurado.

## Mailing

Se integró Resend como servicio gratuito/simple para bajo volumen.

Variables:

```env
RESEND_API_KEY=re_...
MAIL_FROM=PaperMind <noreply@tu-dominio.com>
```

Se usa para:

- Confirmación de pedidos.
- Confirmación de devoluciones.
- Promociones generadas por el agente de fidelización.



## Autorización temporal

Los endpoints protegidos usan headers simples:

```http
x-user-id: UUID_DEL_USUARIO
x-role: cliente | cajero | administrador | propietario
```

Para producción, lo correcto es validar el JWT de Supabase en backend y derivar el rol desde `usuarios_sistema`/`clientes`.

## Ejemplos para Postman

### Variables de entorno en Postman

Crea un environment en Postman con estas variables:

| Variable | Ejemplo |
| --- | --- |
| `base_url` | `http://127.0.0.1:8080` |
| `admin_id` | UUID de un usuario en `usuarios_sistema` con rol administrador |
| `cajero_id` | UUID de un usuario en `usuarios_sistema` con rol cajero |
| `cliente_id` | UUID de un usuario en `clientes` |
| `categoria_id` | UUID de una categoría creada |
| `proveedor_id` | UUID de un proveedor creado |
| `producto_id` | UUID de un producto creado |
| `carrito_id` | UUID del carrito activo, opcional |
| `pedido_id` | UUID de un pedido creado |
| `detalle_pedido_id` | UUID de un detalle de pedido |
| `motivo_id` | UUID de un motivo de devolución |
| `email_prueba` | correo donde quieres recibir pruebas |

### Headers comunes

Para requests públicos:

```http
Content-Type: application/json
```

Para cliente:

```http
Content-Type: application/json
x-role: cliente
x-user-id: {{cliente_id}}
```

Para cajero:

```http
Content-Type: application/json
x-role: cajero
x-user-id: {{cajero_id}}
```

Para administrador:

```http
Content-Type: application/json
x-role: administrador
x-user-id: {{admin_id}}
```

> Nota: el backend usa estos headers como autorización temporal. Para probar endpoints protegidos, el UUID debe existir en Supabase según corresponda.

---

## Salud

### Verificar estado del backend

```http
GET {{base_url}}/health
```

Respuesta esperada:

```json
{
  "servicio": "PaperMind Backend",
  "estado": "ok",
  "openrouter_configurado": true,
  "correo_configurado": true,
  "supabase_configurado": true
}
```

---

## Categorías

### Listar categorías

```http
GET {{base_url}}/api/categorias
```

### Crear categoría

Headers: administrador.

```http
POST {{base_url}}/api/categorias
```

Body:

```json
{
  "nombre": "Papelería básica",
  "categoria_padre_id": null
}
```

Guarda el `id` de la respuesta como `categoria_id`.

---

## Proveedores

### Listar proveedores

Headers: administrador o cajero.

```http
GET {{base_url}}/api/proveedores
```

### Crear proveedor

Headers: administrador.

```http
POST {{base_url}}/api/proveedores
```

Body:

```json
{
  "nombre": "Distribuidora Escolar MX",
  "contacto_nombre": "María López",
  "correo": "proveedor@example.com",
  "telefono": "5512345678",
  "canal_digital": true,
  "prioridad": 10
}
```

Guarda el `id` como `proveedor_id`.

---

## Productos

### Listar productos

```http
GET {{base_url}}/api/productos
```

### Buscar productos por texto

```http
GET {{base_url}}/api/productos?q=cuaderno
```

### Filtrar productos disponibles por categoría

```http
GET {{base_url}}/api/productos?categoria_id={{categoria_id}}&disponible=true
```

### Obtener producto por ID

```http
GET {{base_url}}/api/productos/{{producto_id}}
```

### Crear producto

Headers: administrador.

```http
POST {{base_url}}/api/productos
```

Body:

```json
{
  "nombre": "Cuaderno profesional cuadro chico",
  "descripcion": "Cuaderno de 100 hojas, pasta dura, marca PaperMind",
  "categoria_id": "{{categoria_id}}",
  "precio_venta": "45.00",
  "precio_costo": "28.00",
  "stock_actual": 50,
  "punto_reorden": 10,
  "proveedor_principal_id": "{{proveedor_id}}",
  "proveedor_alternativo_id": null,
  "codigo_barras_qr": "CUAD-001",
  "es_temporada": false,
  "fecha_activacion": null,
  "fecha_desactivacion": null,
  "imagen_url": null
}
```

Guarda el `id` como `producto_id`.

### Actualizar producto

Headers: administrador.

```http
PATCH {{base_url}}/api/productos/{{producto_id}}
```

Body:

```json
{
  "precio_venta": "49.00",
  "stock_actual": 35,
  "punto_reorden": 8
}
```

### Desactivar producto

Headers: administrador.

```http
DELETE {{base_url}}/api/productos/{{producto_id}}
```

### Recomendaciones de producto

```http
GET {{base_url}}/api/productos/{{producto_id}}/recomendaciones
```

### Suscribirse a disponibilidad

Headers: cliente.

```http
POST {{base_url}}/api/productos/{{producto_id}}/suscripciones-disponibilidad
```

Sin body.

---

## Carrito

### Obtener carrito activo

Headers: cliente.

```http
GET {{base_url}}/api/carrito
```

### Agregar producto al carrito

Headers: cliente.

```http
POST {{base_url}}/api/carrito/items
```

Body:

```json
{
  "producto_id": "{{producto_id}}",
  "cantidad": 2
}
```

### Eliminar producto del carrito

Headers: cliente.

```http
DELETE {{base_url}}/api/carrito/items/{{producto_id}}
```

---

## Pedidos digitales

### Crear pedido desde carrito

Headers: cliente.

```http
POST {{base_url}}/api/pedidos/desde-carrito
```

Body con pago por tarjeta simulado:

```json
{
  "metodo_pago": "tarjeta",
  "modalidad_entrega": "domicilio",
  "direccion_entrega": "Av. Instituto Politécnico Nacional 123, CDMX",
  "puntos_utilizados": 0,
  "email_confirmacion": "{{email_prueba}}"
}
```

Body con pago OXXO:

```json
{
  "metodo_pago": "oxxo",
  "modalidad_entrega": "recoleccion",
  "direccion_entrega": null,
  "puntos_utilizados": 0,
  "email_confirmacion": "{{email_prueba}}"
}
```

Métodos de pago válidos:

```text
tarjeta | transferencia | oxxo | efectivo
```

Modalidades válidas:

```text
domicilio | recoleccion | mostrador
```

### Listar pedidos como cliente

Headers: cliente.

```http
GET {{base_url}}/api/pedidos
```

### Listar pedidos como administrador o cajero

Headers: administrador o cajero.

```http
GET {{base_url}}/api/pedidos
```

---

## Punto de venta físico

### Registrar venta física

Headers: cajero o administrador.

```http
POST {{base_url}}/api/pos/ventas
```

Body:

```json
{
  "usuario_sistema_id": "{{cajero_id}}",
  "cliente_id": "{{cliente_id}}",
  "metodo_pago": "efectivo",
  "email_confirmacion": "{{email_prueba}}",
  "items": [
    {
      "producto_id": "{{producto_id}}",
      "cantidad": 1
    }
  ]
}
```

Venta sin cliente vinculado:

```json
{
  "usuario_sistema_id": "{{cajero_id}}",
  "cliente_id": null,
  "metodo_pago": "efectivo",
  "email_confirmacion": null,
  "items": [
    {
      "producto_id": "{{producto_id}}",
      "cantidad": 1
    }
  ]
}
```

---

## Puntos de fidelidad

### Consultar puntos de cliente

Headers: cliente dueño de la cuenta o administrador/cajero.

```http
GET {{base_url}}/api/clientes/{{cliente_id}}/puntos
```

---

## Devoluciones

### Crear devolución

Headers: cliente dueño del pedido o administrador/cajero.

```http
POST {{base_url}}/api/devoluciones
```

Body:

```json
{
  "pedido_id": "{{pedido_id}}",
  "detalle_pedido_id": "{{detalle_pedido_id}}",
  "cliente_id": "{{cliente_id}}",
  "motivo_id": "{{motivo_id}}",
  "tipo": "reembolso",
  "producto_sustituto_id": null,
  "procesado_por": "{{cajero_id}}",
  "email_confirmacion": "{{email_prueba}}"
}
```

Para cambio:

```json
{
  "pedido_id": "{{pedido_id}}",
  "detalle_pedido_id": "{{detalle_pedido_id}}",
  "cliente_id": "{{cliente_id}}",
  "motivo_id": "{{motivo_id}}",
  "tipo": "cambio",
  "producto_sustituto_id": "{{producto_id}}",
  "procesado_por": "{{cajero_id}}",
  "email_confirmacion": "{{email_prueba}}"
}
```

> La devolución solo se acepta si el pedido está dentro de los últimos 7 días naturales.

---

## Agentes IA

### Agente conversacional

Requiere `OPENROUTER_API_KEY`.

```http
POST {{base_url}}/api/agentes/conversacional/mensaje
```

Body:

```json
{
  "cliente_id": "{{cliente_id}}",
  "mensaje": "¿Tienen cuadernos disponibles y cuánto cuestan?"
}
```

Respuesta esperada:

```json
{
  "interaccion_id": "uuid",
  "intencion": "precio",
  "sentimiento": "neutro",
  "escalado": false,
  "motivo_escalado": null,
  "respuesta": "Sí, tenemos cuadernos disponibles...",
  "acciones_sugeridas": ["Agregar producto al carrito"],
  "productos_mencionados": ["Cuaderno profesional cuadro chico 100 hojas"],
  "productos_sugeridos": [
    {
      "id": "uuid",
      "nombre": "Cuaderno profesional cuadro chico 100 hojas",
      "categoria": "Cuadernos",
      "precio_venta": "49.00",
      "stock_actual": 120,
      "estado": "activo"
    }
  ]
}
```

El agente ahora usa productos relevantes según el mensaje, contexto del cliente si se manda `cliente_id`, último pedido del cliente y detección de sentimiento.

Ejemplo fuera de dominio:

```json
{
  "cliente_id": "{{cliente_id}}",
  "mensaje": "¿Puedes ayudarme a hacer mi tarea de matemáticas?"
}
```

### Ejecutar agente de inventario

Headers: administrador o cajero.

```http
POST {{base_url}}/api/agentes/inventario/ejecutar
```

Sin body.

### Ejecutar agente de predicción

Headers: administrador o cajero. Requiere `OPENROUTER_API_KEY`.

```http
POST {{base_url}}/api/agentes/prediccion/ejecutar
```

Sin body.

### Enviar correo de prueba

Headers: administrador. Sirve para diagnosticar Resend sin depender de clientes ni de OpenRouter.

```http
POST {{base_url}}/api/mail/test
```

Body:

```json
{
  "email": "{{email_prueba}}"
}
```

Respuesta exitosa:

```json
{
  "correo_enviado": true,
  "resend": {
    "id": "email_id_de_resend"
  }
}
```

Si Resend rechaza el envío, el backend devuelve el error exacto de Resend.

### Generar promoción personalizada

Headers: administrador o cajero. Requiere `OPENROUTER_API_KEY`. Envía correo si `RESEND_API_KEY` y `MAIL_FROM` están configurados.

```http
POST {{base_url}}/api/agentes/fidelizacion/promocion
```

Body:

```json
{
  "cliente_id": "{{cliente_id}}",
  "email": "{{email_prueba}}"
}
```

La respuesta incluye `resend.id` cuando Resend acepta el correo.

### Configurar parámetro de agente

Headers: administrador.

```http
PUT {{base_url}}/api/agentes/configuracion
```

Body:

```json
{
  "agente": "inventario",
  "parametro": "factor_seguridad_default",
  "valor": "0.20"
}
```

Agentes válidos:

```text
inventario | conversacional | prediccion | fidelizacion | recomendacion
```

---

## Dashboard y reportes

### Dashboard administrativo

Headers: administrador o cajero.

```http
GET {{base_url}}/api/dashboard
```

### Reporte de ventas del día

Headers: administrador o cajero.

```http
GET {{base_url}}/api/reportes/ventas
```

---

## Orden recomendado de pruebas en Postman

1. `GET /health`
2. `POST /api/categorias`
3. `POST /api/proveedores`
4. `POST /api/productos`
5. `GET /api/productos`
6. `GET /api/carrito`
7. `POST /api/carrito/items`
8. `POST /api/pedidos/desde-carrito`
9. `GET /api/clientes/{{cliente_id}}/puntos`
10. `POST /api/agentes/conversacional/mensaje`
11. `POST /api/agentes/prediccion/ejecutar`
12. `GET /api/dashboard`

## Seed de productos

Para cargar productos de prueba en el backend desplegado en Render:

```bash
python scripts/seed_products_api.py
```

Por defecto usa:

```text
https://agentic-paperwork.onrender.com
```

Para apuntar a otra URL:

```bash
API_BASE_URL=http://127.0.0.1:8080 python scripts/seed_products_api.py
```

El script crea categorías, proveedor base y productos escolares como cuadernos, lápices, plumas, hojas, carpetas, arte y oficina.

## Keep-alive para Render

Render puede suspender instancias gratuitas después de un periodo sin tráfico. Se incluye un script simple para hacer ping periódico al endpoint `/health`.

Ejecutar con valores por defecto:

```bash
python scripts/keep_alive.py
```

Por defecto consulta:

```text
https://agentic-paperwork.onrender.com/health
```

Configurar intervalo o URL:

```bash
KEEP_ALIVE_URL=https://agentic-paperwork.onrender.com/health KEEP_ALIVE_INTERVAL=600 python scripts/keep_alive.py
```

En Windows PowerShell:

```powershell
$env:KEEP_ALIVE_URL="https://agentic-paperwork.onrender.com/health"
$env:KEEP_ALIVE_INTERVAL="600"
python scripts/keep_alive.py
```

> Nota: el proceso debe quedarse corriendo en alguna computadora, servidor o servicio de automatización externo. Si cierras la terminal, se detiene.

## Validación

```bash
cargo check
```
