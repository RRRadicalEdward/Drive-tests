use actix_web::{App, HttpServer};
use driving_tests_site::{db, web};
use log::info;

use simple_logger::SimpleLogger;

use std::env;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5050".to_string());

    SimpleLogger::new().init().unwrap();

    db::extablish_connection()?;

    let tls_builder = web::tls_builder()?;
    
    info!("Successfully connected to the DB");

    HttpServer::new(|| {
        App::new()
            .service(web::sing_in)
            .service(web::sing_up)
            .service(web::healthy)
    })
    .bind_openssl(server_addr, tls_builder)?
    .run()
    .await
    .map_err(|err| err.into())
}
