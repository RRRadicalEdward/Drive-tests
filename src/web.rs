use actix_web::{get, post, web, HttpResponse};
use log::{debug, error, info};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};
use serde_json::json;

use crate::{
    db::{self, DbPool},
    model::UserForm,
};

#[post("/user")]
pub async fn sing_up(user: web::Json<UserForm>, pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.name, user.name
    );
    let user = user.into_inner();

    let user_clone = user.clone();
    web::block(move || db::registry_new_user(user_clone, pool))
        .await
        .map_err(|err| {
            error!("Error while registering a new user - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    debug!("Successfully registry {} {} user", user.name, user.second_name);

    let http_response = HttpResponse::Ok().content_type("application/json").json(json!({
        "user_name"  : user.name,
        "second_name": user.second_name,
        "scores"     : "0",
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
    let verify_passed = web::block(move || db::verify_password(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("An error occurred while verifying the user - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let scores = web::block(move || db::get_scores(&user_clone, &pool_clone))
        .await
        .map_err(|err| {
            error!("An error occurred while getting scores - {}", err);
            HttpResponse::InternalServerError().finish()
        })?;

    let http_response = match verify_passed {
        true => {
            debug!("The user passed password verifying");
            HttpResponse::Ok().content_type("application/json").json(json!({
                "user_name"  : user.name.clone(),
                "second_name": user.second_name.to_string(),
                "scores"     : scores.to_string(),
            }))
        }
        false => {
            debug!("The user hasn't passed password verify");
            HttpResponse::NotFound().finish()
        }
    };
    Ok(http_response)
}

#[get("/test")]
pub async fn get_test(pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let test = web::block(move || db::get_test(pool)).await.map_err(|err| {
        error!("get test error - {}", err);
        HttpResponse::InternalServerError().finish()
    })?;

    let answers_json = serde_json::to_value(test.answers.clone()).map_err(|err| {
        error!("failed to parse {} test answers to json - {}", test.id, err);
        HttpResponse::InternalServerError().finish()
    })?;

    let response = HttpResponse::Ok().content_type("application/json").json(json!({
        "description": test.description,
        "answers"    : answers_json,
    }));
    Ok(response)
}

#[get("/test?test_id&answer_id")]
pub async fn check_answer(path: web::Path<(u32, u32)>, pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse> {
    let (test_id, answer_id) = path.0;
    let (check_result, test_level) = web::block(move || db::check_test_answer(test_id, answer_id, &pool))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;

    match check_result {
        true => {
            let scores = test_level.to_scores();
            Ok(HttpResponse::Ok()
                .header("scores", scores.to_string())
                .body("The answer is correct"))
        }
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
    let (check_result, test_level) = web::block(move || db::check_test_answer(test_id, answer_id, &pool_clone))
        .await
        .map_err(|err| {
            error!("check_if_user_exists error - {}", err);
            HttpResponse::InternalServerError().finish();
        })?;

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    match check_result {
        true => {
            let scores = test_level.to_scores();
            web::block(move || db::add_scores(&user_clone, scores, &pool_clone))
                .await
                .map_err(|err| {
                    error!("failed to add new scores after passing a test successfully - {}", err);
                    HttpResponse::InternalServerError().finish();
                })?;
            Ok(HttpResponse::Ok()
                .header("scores", scores.to_string())
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

pub fn tls_builder() -> anyhow::Result<SslAcceptorBuilder> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("cert/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("cert/cert.pem")?;
    Ok(builder)
}
