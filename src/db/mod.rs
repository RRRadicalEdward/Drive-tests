use diesel::{
    expression::dsl::{count, exists},
    insert_into, select,
    sqlite::SqliteConnection,
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;
use rand::rngs::OsRng;
use rsa::{pem, PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

use actix_web::web;

use std::{convert::TryFrom, fs::File, io::Read, ops::Deref};

pub mod model;
pub mod schema;

use schema::{tests, users};

use model::UserForm;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> DbPool {
    let database_url = "../../drive_tests_db";
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to crate DB pool")
}

pub fn registry_new_user(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<()> {
    use self::users::dsl::*;

    if check_if_user_exists(user.clone(), pool.clone())? {
        return Err(anyhow!(format!(
            "{} {} user already exists",
            user.name, user.second_name
        )));
    }

    let encrypted_password = encrypted_password(user.password.clone())?;

    let db = pool.get().unwrap();
    let insert_result = insert_into(users)
        .values((
            name.eq(user.name),
            second_name.eq(user.second_name),
            password.eq(encrypted_password),
        ))
        .execute(db.deref());

    match insert_result {
        Ok(0) => Err(anyhow!("Failed to insert a row to the Users table")),
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn check_if_user_exists(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<bool> {
    use self::users::dsl::*;

    let db = pool.get().unwrap();

    select(exists(
        users.filter((name.eq(user.name)).and(second_name.eq(user.second_name))),
    ))
    .get_result(db.deref())
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

pub fn verify_password(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<(bool, i32)> {
    use self::users::dsl::*;

    let db = pool.get().unwrap();

    let selected_user: model::User = users
        .order(id)
        .filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .map_err(|err| anyhow!("failed to find in the DB User - {}", err))?;

    let decrypted_password = decrypt_password(selected_user.password)?;

    if decrypted_password == user.password {
        Ok((true, selected_user.scores))
    } else {
        Ok((false, 0))
    }
}
pub fn get_test(pool: web::Data<DbPool>) -> anyhow::Result<model::Test> {
    use self::tests::dsl::*;

    let db = pool.get().unwrap();
    let count = tests
        .select(count(id))
        .execute(db.deref())
        .map_err(|err| anyhow!("failed to get tests count - {}", err))?;

    let rand_test = rand::random::<usize>() % count;

    let test = tests
        .order(id)
        .filter(id.eq(rand_test as i32))
        .first::<model::Test>(db.deref())
        .map_err(|err| anyhow!("failed to get rand test - {}", err))?;
    Ok(test)
}

pub fn update_scores(user: &UserForm, new_scores: u32, pool: &web::Data<DbPool>) -> anyhow::Result<()> {
    use self::users::dsl::*;
    let db = pool.get().unwrap();

    let mut selected_user: model::User = users
        .order(id)
        .filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .map_err(|err| anyhow!("failed to find in the DB User - {}", err))?;

    selected_user.scores += new_scores as i32;

    diesel::update(users.filter(id.eq(selected_user.id)))
        .set(scores.eq(selected_user.scores))
        .execute(db.deref())
        .map_err(|err| anyhow!("failed to update user scores - {}", err))?;

    Ok(())
}
