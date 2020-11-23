use actix_web::{get, post, web, HttpResponse};
use serde_derive::{Deserialize, Serialize};
use log::{debug, error, info};

use std::io::{self, ErrorKind};
use std::convert::TryInto;

use crate::db;
use crate::model::UserForm;

#[post("/user")]
pub async fn sing_up(user: web::Json<UserForm>) -> actix_web::Result<HttpResponse> {
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.name, user.name
    );

    let http_response = match db::registar_new_user(&user.name, &user.name, &user.password) {
        Ok(_) => {
            debug!("Successfully registory a new user");
            let response = UserRequestResult::Succesfully(UserGoodResponse {
                user_name: user.name.to_string(),
                second_name: user.second_name.to_string(),
                scores: 0,
            });

            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            error!("Error while registoring a new user - {}", err);
            return Err(actix_web::error::ErrorInternalServerError(io::Error::new(
                ErrorKind::Other,
                "An Error occured in the server, try one more time",
            )));
        }
    };
    info!("Successfully proccess a new user");

    Ok(http_response)
}

#[get("/user")]
pub async fn sing_in(user: web::Json<UserForm>) -> actix_web::Result<HttpResponse> {
    debug!(
        "Validation of user:[name - {}, second name - {}]",
        user.name, user.second_name
    );
    match db::check_if_user_exists(&user.name, &user.second_name) {
        Ok(true) => {}
        Ok(false) => {
            debug!("The user doesn't present in the DB");
            let response = UserRequestResult::Failed(UserBadReponse {
                cause: String::from("The user doesn't present in the DB"),
            });
            return Ok(HttpResponse::NotFound().json(response));
        }
        Err(err) => {
            error!("An error occured while singing in the user - {}", err);
            return Err(actix_web::error::ErrorInternalServerError(io::Error::new(
                ErrorKind::Other,
                "An Error occured in the server, try one more time",
            )));
        }
    };

    debug!("The user exists in the DB");

    let http_response =
        match db::verify_password(&user.name, &user.second_name, user.password.to_string()) {
            Ok((true, scores)) => {
                debug!("The user passed password verifying");
                let user = UserGoodResponse {
                    user_name: user.name.clone(),
                    second_name: user.second_name.to_string(),
                    scores: scores.try_into().unwrap(),
                };
                HttpResponse::Ok().json(user)
            }
            Ok((false, _)) => {
                debug!("The user hasn't passed password verify");
                let repsonse = UserRequestResult::Failed(UserBadReponse {
                    cause: String::from("The user hasn't passed password verify"),
                });
                HttpResponse::NotFound().json(repsonse)
            }
            Err(err) => {
                error!("An error occured while verifying the user  - {}", err);

                return Err(actix_web::error::ErrorInternalServerError(io::Error::new(
                    ErrorKind::Other,
                    "An Error occured in the server, try one more time",
                )));
            }
        };
    Ok(http_response)
}
#[derive(Serialize, Deserialize)]
pub struct UserGoodResponse {
    user_name: String,
    second_name: String,
    scores: u32,
}
#[derive(Serialize, Deserialize)]
pub struct UserBadReponse {
    cause: String,
}
#[derive(Serialize, Deserialize)]
pub enum UserRequestResult {
    Succesfully(UserGoodResponse),
    Failed(UserBadReponse),
}
