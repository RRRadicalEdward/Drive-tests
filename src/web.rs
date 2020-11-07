use log::info;

use rocket::http::RawStr;
use rocket::http::{ContentType, Status};
use rocket::request::{LenientForm, Request};
use rocket::response::{self, Responder, Response};

use crate::db;

#[get("/show_users_table")]
pub fn show_users_table() -> &'static str {
    "hello world"
}

#[post("/registor_new_user", data = "<user>")]
pub fn registor_new_user(user: LenientForm<User>) -> anyhow::Result<()> {
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.user_name, user.second_name
    );

    db::registar_new_user(&user.user_name, &user.second_name, user.password)?;

    info!("Successfully registary a new user");
    Ok(())
}

#[post("/validate_user", data = "<user>")]
pub fn validate_user(user: LenientForm<User>) -> anyhow::Result<UserValidationResult> {
    info!(
        "validation of user:[name - {}, second name - {}]",
        user.user_name, user.second_name
    );
    let check_result = db::check_if_user_exists(&user.user_name, &user.second_name)?;
    info!("Validation result - {:?}", check_result);
    Ok(UserValidationResult(check_result))
}

#[derive(FromForm)]
pub struct User<'a> {
    user_name: &'a RawStr,
    second_name: &'a RawStr,
    password: &'a RawStr,
}

#[derive(Debug)]
pub struct UserValidationResult(bool);

impl<'r> Responder<'r> for UserValidationResult {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .status(Status::Ok)
            .header(ContentType::Plain)
            .raw_header("UserValidationResult", format!("{:?}", self.0))
            .ok()
    }
}
