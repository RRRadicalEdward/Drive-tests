use log::{debug, error, info};

use rocket::http::RawStr;
use rocket::http::{ContentType, Status};
use rocket::request::{LenientForm, Request};
use rocket::response::{self, Responder, Response};

//use diesel::result::{DatabaseErrorKind, Error};

use std::convert::TryInto;

use crate::db;

#[post("/sing_up", data = "<user>")]
pub fn sing_up(user: LenientForm<UserRequestForm>) -> UserRequestResult {
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.user_name, user.second_name
    );

    let response = match db::registar_new_user(&user.user_name, &user.second_name, user.password) {
        Ok(_) => {
            debug!("Successfully registory a new user");
            UserRequestResult::Succesfully(UserGoodResponse {
                user_name: user.user_name.to_string(),
                second_name: user.second_name.to_string(),
                scores: 0,
            })
        }
        Err(err) => {
            error!("Error while registoring a new user - {}", err);
            UserRequestResult::Failed(UserBadReponse {
                cause: String::from("error!"),
            })
        }
    };
    info!("Successfully proccess a new user");

    response
}

#[post("/sing_in", data = "<user>")]
pub fn sing_in(user: LenientForm<UserRequestForm>) -> UserRequestResult {
    debug!(
        "Validation of user:[name - {}, second name - {}]",
        user.user_name, user.second_name
    );
    match db::check_if_user_exists(&user.user_name, &user.second_name) {
        Ok(true) => {}
        Ok(false) => {
            debug!("The user doesn't present in the DB");
            return UserRequestResult::Failed(UserBadReponse {
                cause: String::from("The user doesn't present in the DB"),
            });
        }
        Err(err) => {
            error!("Error occured while singing in the user - {}", err);
            return UserRequestResult::Failed(UserBadReponse {
                cause: String::from("error!"),
            });
        }
    };

    debug!("The user exists in the DB");

    match db::verify_password(
        &user.user_name,
        &user.second_name,
        user.password.to_string(),
    ) {
        Ok((true, scores)) => {
            debug!("The user passed password verifying");
            UserRequestResult::Succesfully(UserGoodResponse {
                user_name: user.user_name.to_string(),
                second_name: user.second_name.to_string(),
                scores: scores.try_into().unwrap(),
            })
        }
        Ok((false, _)) => {
            debug!("The user hasn't passed password verify");
            UserRequestResult::Failed(UserBadReponse {
                cause: String::from("The user hasn't passed password verify"),
            })
        }
        Err(err) => {
            error!("Error occured while veriying the user  - {}", err);
            UserRequestResult::Failed(UserBadReponse {
                cause: String::from("error!"),
            })
        }
    }
}

#[derive(FromForm)]
pub struct UserRequestForm<'a> {
    user_name: &'a RawStr,
    second_name: &'a RawStr,
    password: &'a RawStr,
}
pub struct UserGoodResponse {
    user_name: String,
    second_name: String,
    scores: u32,
}

pub struct UserBadReponse {
    cause: String,
}

pub enum UserRequestResult {
    Succesfully(UserGoodResponse),
    Failed(UserBadReponse),
}

impl<'r> Responder<'r> for UserGoodResponse {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .status(Status::Ok)
            .header(ContentType::Plain)
            .raw_header("user_name", format!("{}", self.user_name))
            .raw_header("second_name", format!("{}", self.second_name))
            .raw_header("scores", format!("{}", self.scores))
            .ok()
    }
}

impl<'r> Responder<'r> for UserBadReponse {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        Response::build()
            .status(Status::BadRequest)
            .header(ContentType::Plain)
            .raw_header("Failed to procces the request", self.cause)
            .ok()
    }
}

impl<'r> Responder<'r> for UserRequestResult {
    fn respond_to(self, r: &Request) -> response::Result<'r> {
        match self {
            UserRequestResult::Succesfully(user_good_reponse) => user_good_reponse.respond_to(r),
            UserRequestResult::Failed(user_bad_response) => user_bad_response.respond_to(r),
        }
    }
}
