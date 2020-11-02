use rsa::{pem, PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

use rand::rngs::OsRng;

use diesel::mysql::MysqlConnection;
use diesel::prelude::Connection;

use serde_json::Value;

use log::{debug, info};

use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, ErrorKind, Read};

enum TestLevel {
    Easy,
    Medium,
    High,
}

impl TestLevel {
    fn new(level_value: u32) -> Result<TestLevel, io::Error> {
        match level_value {
            1 => Ok(TestLevel::Easy),
            3 => Ok(TestLevel::Medium),
            5 => Ok(TestLevel::High),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Got incorrect test level value",
            )),
        }
    }
}

pub fn extablish_connection() -> Result<MysqlConnection, diesel::ConnectionError> {
    let config_file = File::open("config.json").unwrap();

    let config: Value = serde_json::from_reader(config_file).unwrap();

    let user_name = config["user_name"].as_str().unwrap();
    let password = config["password"].as_str().unwrap();
    let localhost = "127.0.0.1:3306";
    let db_name = config["db_name"].as_str().unwrap();

    let database_url = format!(
        "mysql://{}:{}@{}/{}",
        user_name, password, localhost, db_name
    );
    MysqlConnection::establish(&database_url)
}

pub fn registary_new_user(name: &str, second_name: &str, password: &str) -> anyhow::Result<()> {
    debug!(
        "There is a new user:[name - {}, second name - {}]",
        name, second_name
    );
    let mut rng = OsRng;

    let mut public_key_file = File::open("public-key.pem")?;
    let mut buffer = String::new();
    public_key_file.read_to_string(&mut buffer)?;

    let pem = pem::parse(buffer.into_bytes())?;

    let public_key = RSAPublicKey::try_from(pem)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let _encrypted_password = public_key.encrypt(&mut rng, padding, password.as_bytes())?;

    info!("Successfully registered a new user");

    Ok(())
}
