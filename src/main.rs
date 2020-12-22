use actix_web::{
    http,
    middleware::{self, Logger},
    App, HttpServer,
};
use log::info;
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use std::env;

use lib::db::{establish_connection, DEFAULT_DATABASE_URL};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    if env::var("DATABASE_URL").is_err() {
        env::set_var("DATABASE_URL", DEFAULT_DATABASE_URL);
    }

    let server_addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:5050".to_string());

    info!("Running server on {}", server_addr);

    let connection_pool = establish_connection();

    let tls_builder = tls_builder()?;

    info!(
        "Successfully connected to the DB on {}",
        env::var("DATABASE_URL").unwrap()
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %t %r %b %s %T"))
            .wrap(middleware::Compress::new(http::ContentEncoding::Identity))
            .data(connection_pool.clone())
            .configure(services_config)
    })
    .bind_openssl(server_addr, tls_builder)?
    .run()
    .await
    .map_err(|err| err.into())
}

pub fn services_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(lib::sing_up)
        .service(lib::sing_in)
        .service(lib::check_answer_with_user)
        .service(lib::get_test)
        .service(lib::check_answer)
        .service(lib::healthy);
}

pub fn tls_builder() -> anyhow::Result<SslAcceptorBuilder> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("cert/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert/cert.pem")?;
    Ok(builder)
}
