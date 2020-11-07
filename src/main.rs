#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use log::info;
use simple_logger::SimpleLogger;

use driving_tests_site::{db, web};

fn main() -> anyhow::Result<()> {
    SimpleLogger::new().init().unwrap();

    db::extablish_connection()?;

    info!("Successfully connected to the DB");

    rocket::ignite()
        .mount("/", routes![web::show_users_table, web::registor_new_user, web::validate_user])
        .launch();
    Ok(())
}
