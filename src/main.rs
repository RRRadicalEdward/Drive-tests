use actix_web::{App, HttpServer};
use log::info;
use simple_logger::SimpleLogger;

use driving_tests_site::{db, web};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::new().init().unwrap();

    db::extablish_connection()?;

    info!("Successfully connected to the DB");

    HttpServer::new(|| App::new().service(web::sing_in).service(web::sing_up))
        .bind("127.0.0.1:5050")?
        .run()
        .await
        .map_err(|err| err.into())
}
