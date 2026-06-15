mod auth;
mod config;
mod error;
mod handlers;
mod models;
mod services;
mod state;

use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, web::Data};
use reqwest::Client;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;

use crate::{config::Config, state::AppState};

#[derive(Serialize)]
struct HealthResponse {
    servicio: &'static str,
    estado: &'static str,
    version: &'static str,
    openrouter_configurado: bool,
    correo_configurado: bool,
    supabase_configurado: bool,

    modulos: Vec<&'static str>,
}

#[get("/health")]
async fn health(state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        servicio: "PaperMind Backend",
        estado: "ok",
        version: env!("CARGO_PKG_VERSION"),
        openrouter_configurado: state.config.openrouter_enabled(),
        correo_configurado: state.config.mail_enabled(),
        supabase_configurado: state.config.supabase_url.is_some()
            && state.config.supabase_anon_key.is_some(),

        modulos: vec![
            "catalogo",
            "carrito",
            "pedidos",
            "pagos",
            "inventario",
            "proveedores",
            "fidelizacion",
            "devoluciones",
            "agente_conversacional_openrouter",
            "agente_inventario",
            "agente_prediccion_openrouter",
            "mailing_resend",
            "reportes",
        ],
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&config.database_url)
        .await
        .expect("no se pudo conectar a DATABASE_URL");

    let state = AppState {
        pool,
        config: config.clone(),
        http: Client::new(),
    };

    println!(
        "PaperMind backend corriendo en http://{}:{}",
        config.host, config.port
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(Data::new(state.clone()))
            .service(health)
            .configure(handlers::catalog::config)
            .configure(handlers::providers::config)
            .configure(handlers::orders::config)
            .configure(handlers::agents::config)
    })
    .bind((config.host, config.port))?
    .run()
    .await
}
