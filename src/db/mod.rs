use diesel::{
    connection::SimpleConnection,
    expression::dsl::{count, exists},
    insert_into,
    r2d2::{ConnectionManager, CustomizeConnection},
    select,
    sqlite::SqliteConnection,
    BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use r2d2::Pool;
use rand::rngs::OsRng;
use rsa::{pem, PaddingScheme, PublicKey, RSAPrivateKey, RSAPublicKey};

use actix_web::web;

use std::{
    convert::{TryFrom, TryInto},
    fs::File,
    io::Read,
    ops::Deref,
    time::Duration,
};

pub mod model;
pub mod schema;

use crate::db::model::TestLevel;
use model::UserForm;
use schema::{tests, users};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug)]
struct ConnectionCustomizer {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for ConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        let mut command = String::new();
        if self.enable_wal {
            command.push_str(
                "PRAGMA journal_mode = WAL;
                          PRAGMA synchronous = NORMAL;",
            );
        }
        if self.enable_foreign_keys {
            command.push_str("PRAGMA foreign_keys = ON;")
        }
        if let Some(d) = self.busy_timeout {
            command.push_str(&format!("PRAGMA busy_timeout = {};", d.as_millis()))
        }
        conn.batch_execute(&command).map_err(diesel::r2d2::Error::QueryError)?;
        Ok(())
    }
}

pub fn establish_connection() -> DbPool {
    let database_path = "drive_tests_db.db";
    let manager = ConnectionManager::<SqliteConnection>::new(database_path);
    r2d2::Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionCustomizer {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(5)),
        }))
        .build(manager)
        .expect("Failed to crate DB pool")
}

pub fn registry_new_user(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<()> {
    use schema::users::dsl::*;

    if check_if_user_exists(user.clone(), pool.clone())? {
        return Err(anyhow!(format!(
            "{} {} user already exists",
            user.name, user.second_name
        )));
    }

    let encrypted_password = encrypt_password(user.password.clone())?;

    let db = pool.get().unwrap();
    let insert_result = insert_into(users)
        .values(&(
            name.eq(user.name),
            second_name.eq(user.second_name),
            password.eq(encrypted_password),
            scores.eq(0),
        ))
        .execute(db.deref());

    match insert_result {
        Ok(0) => Err(anyhow!("Failed to insert a row to the Users table")),
        Ok(_) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

pub fn check_if_user_exists(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<bool> {
    use schema::users::dsl::*;

    let db = pool.get().unwrap();

    select(exists(
        users.filter((name.eq(user.name)).and(second_name.eq(user.second_name))),
    ))
    .get_result(db.deref())
    .map_err(|err| err.into())
}

fn encrypt_password(password: String) -> anyhow::Result<String> {
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
        encrypted_password_hex.push_str(&format!("{:02x}", num));
    });
    Ok(encrypted_password_hex)
}

fn decrypt_password(encrypted_password_hex: String) -> anyhow::Result<String> {
    let mut encrypted_password: Vec<u8> = vec![0; encrypted_password_hex.len() / 2];

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

pub fn verify_password(user: UserForm, pool: web::Data<DbPool>) -> anyhow::Result<bool> {
    use self::users::dsl::*;

    let db = pool.get().unwrap();

    let selected_user: model::User = users
        .order(id)
        .filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .map_err(|err| {
            anyhow!(
                "failed to find in {} {} user in the DB - {}",
                user.name,
                user.second_name,
                err
            )
        })?;

    let decrypted_password = decrypt_password(selected_user.password)?;
    Ok(decrypted_password == user.password)
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

pub fn add_scores(user: &UserForm, add_scores: u32, pool: &web::Data<DbPool>) -> anyhow::Result<()> {
    use self::users::dsl::*;
    let db = pool.get().unwrap();

    let mut selected_user: model::User = users
        .order(id)
        .filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .map_err(|err| {
            anyhow!(
                "failed to find in {} {} user in the DB - {}",
                user.name,
                user.second_name,
                err
            )
        })?;

    selected_user.scores += add_scores as i32;

    diesel::update(users.filter(id.eq(selected_user.id)))
        .set(scores.eq(selected_user.scores))
        .execute(db.deref())
        .map_err(|err| {
            anyhow!(
                "failed to find in {} {} user in the DB - {}",
                user.name,
                user.second_name,
                err
            )
        })?;

    Ok(())
}

pub fn get_scores(user: &UserForm, pool: &web::Data<DbPool>) -> anyhow::Result<u32> {
    use self::users::dsl::*;
    let db = pool.get().unwrap();

    let selected_user: model::User = users
        .order(id)
        .filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .map_err(|err| {
            anyhow!(
                "failed to find in {} {} user in the DB - {}",
                user.name,
                user.second_name,
                err
            )
        })?;
    Ok(selected_user.scores.try_into()?)
}
pub fn remove_user_from_db(user: UserForm, pool: &web::Data<DbPool>) {
    use self::users::dsl::*;

    let db = pool.get().unwrap();
    diesel::delete(users.filter((name.eq(user.name.clone())).and(second_name.eq(user.second_name))))
        .execute(db.deref())
        .unwrap();
}
pub fn check_test_answer(test_id: u32, answer_id: u32, pool: &web::Data<DbPool>) -> anyhow::Result<(bool, TestLevel)> {
    use self::tests::dsl::*;
    let db = pool.get().unwrap();

    let test_id: i32 = test_id.try_into()?;
    let answer_id: i32 = answer_id.try_into()?;

    let selected_test: model::Test = tests
        .order(id)
        .filter(id.eq(test_id))
        .first::<model::Test>(db.deref())
        .map_err(|err| anyhow!("failed to select test with {} id: {}", test_id, err))?;
    let test_level = TestLevel::new(selected_test.level.try_into()?)?;
    Ok((selected_test.right_answer_id == answer_id, test_level))
}

#[cfg(test)]
mod _tests {
    use super::*;
    use lazy_static::lazy_static;
    use uuid::Uuid;

    const PASSWORD: &str = "password";

    lazy_static! {
        static ref DB: DbPool = establish_connection();
    }

    fn generate_rand_user() -> UserForm {
        let name = Uuid::new_v4().to_string();
        let second_name = Uuid::new_v4().to_string();
        let password = PASSWORD.to_string();
        UserForm {
            name,
            second_name,
            password,
        }
    }

    #[test]
    fn connection_to_db() {
        assert!(DB.get().is_ok());
    }

    #[test]
    fn encrypted_decrypt_password() {
        let password = "password".to_string();

        let encrypted_password = encrypt_password(password.clone()).unwrap();
        let decrypted_password = decrypt_password(encrypted_password).unwrap();

        assert_eq!(password, decrypted_password);
    }

    #[test]
    fn registry_new_user_test() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        let registry_result = registry_new_user(user.clone(), db.clone());

        remove_user_from_db(user, &db);
        assert!(registry_result.is_ok());
    }

    #[test]
    fn check_if_user_exist_for_existing_user() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        registry_new_user(user.clone(), db.clone()).unwrap();

        let check_result = check_if_user_exists(user.clone(), db.clone()).unwrap();

        remove_user_from_db(user, &db);
        assert!(check_result);
    }

    #[test]
    fn check_if_user_exist_for_not_existing_user() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        let check_result = check_if_user_exists(user, db).unwrap();

        assert!(!check_result);
    }

    #[test]
    fn verify_password_for_correct_password() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        registry_new_user(user.clone(), db.clone()).unwrap();

        let verify_password_result = verify_password(user.clone(), db.clone()).unwrap();

        remove_user_from_db(user, &db);
        assert!(verify_password_result);
    }

    #[test]
    fn verify_password_for_incorrect_password() {
        let db = web::Data::new(DB.clone());
        let mut user = generate_rand_user();

        registry_new_user(user.clone(), db.clone()).unwrap();

        user.password = "Some incorrect password".to_string();
        let verify_password_result = verify_password(user.clone(), db.clone()).unwrap();

        remove_user_from_db(user, &db);
        assert!(!verify_password_result);
    }

    #[test]
    fn get_scores_for_just_registered_user() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        registry_new_user(user.clone(), db.clone()).unwrap();

        let scores = get_scores(&user, &db).unwrap();

        remove_user_from_db(user, &db);
        assert_eq!(scores, 0);
    }

    #[test]
    fn get_scores_for_after_add_scores_returns_right_scores_value() {
        let db = web::Data::new(DB.clone());
        let user = generate_rand_user();

        registry_new_user(user.clone(), db.clone()).unwrap();

        let rand_scores = rand::random::<u32>() % 1000;
        add_scores(&user, rand_scores, &db).unwrap();

        let scores = get_scores(&user, &db).unwrap();

        remove_user_from_db(user, &db);
        assert_eq!(scores, rand_scores);
    }
}
