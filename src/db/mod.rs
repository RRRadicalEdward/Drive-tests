use rsa::{pem, PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

use rand::rngs::OsRng;

use diesel::expression::dsl::exists;
use diesel::sqlite::SqliteConnection;
use diesel::QueryDsl;
use diesel::{insert_into, select, Connection};
use diesel::{BoolExpressionMethods, ExpressionMethods, RunQueryDsl};

use std::convert::TryFrom;
use std::fs::File;
use std::io::{self, Read};
use std::string::String;
use std::sync::{Arc, Mutex};

pub mod model;
pub mod schema;

use schema::users;

lazy_static! {
    static ref DB: Arc<Mutex<anyhow::Result<SqliteConnection>>> = {
        let db = extablish_connection_impl();
        Arc::new(Mutex::new(db))
    };
}

pub fn extablish_connection() -> anyhow::Result<()> {
    let db = Arc::clone(&DB);

    let db = &*db.lock().unwrap();
    match db {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::NotConnected,
            format!("Failed to connect to the DataBase due to: {:?}", err),
        )
        .into()),
    }
}

fn extablish_connection_impl() -> anyhow::Result<SqliteConnection> {
    SqliteConnection::establish("../../drive_tests_db").map_err(|err| err.into())
}

pub fn registar_new_user(
    user_name: &str,
    user_second_name: &str,
    user_password: &str,
) -> anyhow::Result<()> {
    use self::users::dsl::*;

    if check_if_user_exists(user_name, user_second_name)? {
        return Err(anyhow!(format!(
            "{} {} user exists",
            user_name, user_second_name
        )));
    }

    let encrypted_password = encrypted_password(user_password.to_string())?;

    let db = Arc::clone(&DB);
    let db = &*db.lock().unwrap();

    let insert_result = insert_into(users)
        .values((
            name.eq(user_name.to_string()),
            second_name.eq(user_second_name.to_string()),
            password.eq(encrypted_password),
        ))
        .execute(db.as_ref().unwrap());

    match insert_result {
        Ok(0) => Err(anyhow!("Failed to insert a row to the Users table")),
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn check_if_user_exists(user_name: &str, user_second_name: &str) -> anyhow::Result<bool> {
    use self::users::dsl::*;
    let db = Arc::clone(&DB);
    let db = db.lock().unwrap();

    select(exists(users.filter(
        (name.eq(user_name)).and(second_name.eq(user_second_name)),
    )))
    .get_result(db.as_ref().unwrap())
    .map_err(|err| err.into())
}

fn encrypted_password(password: String) -> anyhow::Result<String> {
    let mut rng = OsRng;

    let mut public_key_file = File::open("public-key.pem")?;
    let mut buffer = String::new();
    public_key_file.read_to_string(&mut buffer)?;

    let pem = pem::parse(buffer.into_bytes())?;

    let public_key = RSAPublicKey::try_from(pem)?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let encrypted_password = public_key.encrypt(&mut rng, padding, &password.into_bytes())?;
    let mut encrypted_password_hex: String = String::with_capacity(encrypted_password.len() * 2);
    encrypted_password.iter().for_each(|&num| {
        encrypted_password_hex.push_str(&format!("{:x}", num));
    });
    Ok(encrypted_password_hex)
}

fn decrypt_password(encrypted_password_hex: String) -> anyhow::Result<String> {
    let mut encrypted_password: Vec<u8> = Vec::with_capacity(encrypted_password_hex.len() / 2);

    hex::decode_to_slice(encrypted_password_hex, &mut encrypted_password)?;

    let mut private_key_file = File::open("private-key.pem")?;
    let mut buffer = String::new();
    private_key_file.read_to_string(&mut buffer)?;

    let pem = pem::parse(buffer.into_bytes())?;
    let padding = PaddingScheme::new_pkcs1v15_encrypt();

    let private_key = RSAPrivateKey::try_from(pem)?;

    let decrypted_password = private_key.decrypt(padding, &encrypted_password)?;
    let decrypted_password = String::from_utf8(decrypted_password)?;

    Ok(decrypted_password)
}

pub fn verify_password(
    user_name: &str,
    user_second_name: &str,
    user_password: String,
) -> anyhow::Result<(bool, i32)> {
    use self::users::dsl::*;
    let db = Arc::clone(&DB);
    let db = db.lock().unwrap();

    let selected_user: model::User = users
        .order(id)
        .filter((name.eq(user_name)).and(second_name.eq(user_second_name)))
        .first::<model::User>(db.as_ref().unwrap())
        .map_err(|err| anyhow!("failed to find in the DB User - {}", err))?;

    let decrypted_password = decrypt_password(selected_user.password)?;

    if decrypted_password == user_password {
        Ok((true, selected_user.scores))
    } else {
        Ok((false, 0))
    }
}
