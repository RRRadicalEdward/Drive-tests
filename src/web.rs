use actix_web::{get, post, web, HttpResponse};
use log::{debug, error, info};
use serde_json::json;

use crate::{
    db::{self, DbPool},
    model::UserForm,
};

const SCORES_FOR_RIGHT_ANSWER: u32 = 5;

#[post("/user")]
pub async fn sing_up(user: web::Json<UserForm>, pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let user = user.into_inner();
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.name, user.name
    );

    let user_clone = user.clone();
    web::block(move || db::registry_new_user(user_clone, pool)) // TODO: return the user already exist if ethe user already exist in the DB
        .await
        .map_err(|err| {
            error!("Error while registering a new user - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    debug!("Successfully registry {} {} user", user.name, user.second_name);

    let http_response = HttpResponse::Created().content_type("application/json").json(json!({
        "name" : user.name,
        "second_name" : user.second_name,
        "scores" : "0"
    }));

    Ok(http_response)
}

#[get("/user")]
pub async fn sing_in(user: web::Json<UserForm>, pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let user = user.into_inner();
    debug!(
        "Validation of user:[name - {}, second name - {}]",
        user.name, user.second_name
    );

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let check_passed = web::block(move || db::check_if_user_exists(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("An error occurred while singing in the user - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    if !check_passed {
        debug!("The user doesn't present in the DB");
        return Ok(HttpResponse::NotFound().finish());
    }

    debug!("The {} {} user exists in the DB", user.name, user.second_name);
    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let verify_password_passed = web::block(move || db::verify_password(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("An error occurred while verifying the user - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    if !verify_password_passed {
        debug!("The user hasn't passed password verify");
        return Ok(HttpResponse::Forbidden().finish());
    }
    debug!("The user passed password verifying");

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let scores = web::block(move || db::get_scores(&user_clone, &pool_clone))
        .await
        .map_err(|err| {
            error!("An error occurred while getting scores - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    let http_response = HttpResponse::Found().content_type("application/json").json(json!({
       "name"  : user.name.clone(),
       "second_name": user.second_name.to_string(),
       "scores"     : scores.to_string(),
    }));

    Ok(http_response)
}

#[get("/test")]
pub async fn get_test(pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let test = web::block(move || db::get_test(pool)).await.map_err(|err| {
        error!("get test error - {}", err);
        HttpResponse::InternalServerError().finish()
    })?;

    let response = HttpResponse::Ok().content_type("application/json").json(json!({
        "description": test.description,
        "answers"    : test.answers,

    }));
    Ok(response)
}

#[get("/test?test_id&answer_id")]
pub async fn check_answer(path: web::Path<(u32, u32)>, pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let (test_id, answer_id) = path.0;
    let check_result = web::block(move || db::check_test_answer(test_id, answer_id, &pool))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;
    match check_result {
        true => Ok(HttpResponse::Ok()
            .header("scores", SCORES_FOR_RIGHT_ANSWER.to_string())
            .body("The answer is correct")),
        false => Ok(HttpResponse::Ok().header("scores", "0").body("The answer is correct")),
    }
}

#[post("/test/{user}?test_id&answer_id")]
pub async fn check_answer_with_user(
    user: web::Json<UserForm>,
    path: web::Path<(u32, u32)>,
    pool: web::Data<DbPool>,
) -> actix_web::Result<HttpResponse> {
    let (test_id, answer_id) = path.0;

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let check_passed = web::block(move || db::check_if_user_exists(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;

    if !check_passed {
        return Ok(HttpResponse::NotFound().finish());
    }

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let verify_passed = web::block(move || db::verify_password(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;

    if !verify_passed {
        return Ok(HttpResponse::NotFound().finish());
    }

    let pool_clone = pool.clone();
    let check_result = web::block(move || db::check_test_answer(test_id, answer_id, &pool_clone))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;

    let user_clone = user.clone();
    let pool_clone = pool.clone();

    match check_result {
        true => {
            web::block(move || db::add_scores(&user_clone, SCORES_FOR_RIGHT_ANSWER, &pool_clone))
                .await
                .map_err(|err| {
                    error!("failed to add new scores after passing a test successfully - {}", err);
                    HttpResponse::InternalServerError().finish();
                })?;
            Ok(HttpResponse::Ok()
                .header("scores", SCORES_FOR_RIGHT_ANSWER.to_string())
                .body("The answer is correct"))
        }
        false => Ok(HttpResponse::Ok().header("scores", "0").body("The answer is correct")),
    }
}

#[get("/healthy")]
pub async fn healthy() -> actix_web::Result<HttpResponse> {
    let response = "Drive-tests is working and healthy".to_string();
    Ok(HttpResponse::Ok().json(response))
}
