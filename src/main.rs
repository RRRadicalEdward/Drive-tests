use actix_web::{
    http,
    middleware::{self, Logger},
    App, HttpServer,
};
use log::info;

use std::env;

use lib::{db, web};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let server_addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:5050".to_string());

    env_logger::init();

    let connection_pool = db::establish_connection();

    let tls_builder = web::tls_builder()?;

    info!("Successfully connected to the DB");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%a %t %r %b %s %T"))
            .wrap(middleware::Compress::new(http::ContentEncoding::Identity))
            .data(connection_pool.clone())
            .route("/user", actix_web::web::post().to(web::sing_up))
           .service(web::sing_in)
          //  .service(web::sing_up)
            .service(web::check_answer_with_user)
            .service(web::get_test)
            .service(web::check_answer)
            .service(web::healthy)
    })
     //.bind_openssl(server_addr, tls_builder)?
        .bind(server_addr)?
        .run()
    .await
    .map_err(|err| err.into())
}
