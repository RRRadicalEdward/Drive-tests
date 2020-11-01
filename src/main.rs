//use diesel::pg::PgConnection;
use anyhow::Context;
use diesel::mysql::MysqlConnection;
use diesel::prelude::Connection;
fn main() {
    let _db = extablish_connection();
}


pub fn extablish_connection() -> MysqlConnection {
    let database_url = "mysql://root:sashayusuk@127.0.0.1:3306/world";

    MysqlConnection::establish(&database_url).context("Failed to connect to MySql data base").unwrap()
}
