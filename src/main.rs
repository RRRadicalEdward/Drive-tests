use actix_web::{
    http,
    middleware::{self, Logger},
    App, HttpServer,
};
use log::{error, info};
use std::{env, path::Path};

use lib::{
    db::{establish_connection, insert_tests_to_db},
    utils,
};

const DEFAULT_IP_ADDR: &str = "127.0.0.1:5050";

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let server_addr = env::var("SERVER_IP_ADDR").unwrap_or_else(|_| DEFAULT_IP_ADDR.to_string());

    info!("Running server on {}", server_addr);

    let connection_pool = establish_connection();

    if env::args().nth(1).is_some() {
        let path_to_tests = env::args().nth(1).unwrap();
        let path = Path::new(path_to_tests.as_str());

        match path.exists() {
            true => {
                let _ = insert_tests_to_db(&path, &connection_pool)
                    .map_err(|err| error!("Insert values to the DB failed due to: {}", err));
            }
            false => error!("{} path to the tests isn't valid", path_to_tests),
        }
    }

    let tls_builder = utils::tls_builder()?;

    info!(
        "Successfully connected to the DB on {}",
        env::var("DATABASE_URL").unwrap()
    );

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %t %r %b %s %T"))
            .wrap(middleware::Compress::new(http::ContentEncoding::Identity))
            .data(connection_pool.clone())
            .configure(utils::services_config)
    })
    .bind_openssl(server_addr, tls_builder)?
    .run()
    .await
    .map_err(|err| err.into())
}
