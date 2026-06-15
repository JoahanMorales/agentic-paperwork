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

## Endpoints principales

### Salud

- `GET /health`

### Catálogo

- `GET /api/categorias`
- `POST /api/categorias`
- `GET /api/productos`
- `GET /api/productos/{id}`
- `POST /api/productos`
- `PATCH /api/productos/{id}`
- `DELETE /api/productos/{id}`
- `GET /api/productos/{id}/recomendaciones`
- `POST /api/productos/{id}/suscripciones-disponibilidad`

### Carrito y ventas

- `GET /api/carrito`
- `POST /api/carrito/items`
- `DELETE /api/carrito/items/{producto_id}`
- `POST /api/pedidos/desde-carrito`
- `GET /api/pedidos`
- `POST /api/pos/ventas`

### Administración

- `GET /api/proveedores`
- `POST /api/proveedores`
- `GET /api/dashboard`
- `GET /api/reportes/ventas`
- `PUT /api/agentes/configuracion`

## Validación

```bash
cargo check
```
