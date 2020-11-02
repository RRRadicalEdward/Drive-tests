use log::{error, info};
use simple_logger::SimpleLogger;

use driving_tests_site::db;

fn main() -> anyhow::Result<()> {
    SimpleLogger::new().init().unwrap();

    let db = db::extablish_connection();

    let _db = match db {
        Ok(db) => db,
        Err(err) => {
            error!("Failed to connect to the DataBase due to: {:?}", err);
            return Err(err.into());
        }
    };

    info!("Successfully connected to the DataBAse");

    db::registary_new_user("Sasha", "Yusuk", "mypassword")?;
    Ok(())
}
