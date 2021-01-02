use actix_web::{
    get, post,
    web::{block, Data, Json, Query},
    HttpResponse, Result,
};
use image::EncodableLayout;
use log::{debug, error, info};
use openssl::base64;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    db::{self, DbPool},
    model::UserForm,
};

const SCORES_FOR_RIGHT_ANSWER: u32 = 5;

#[derive(Deserialize, Serialize)]
pub struct AnswerForm {
    pub test_id: u32,
    pub answer_id: u32,
}

#[derive(Deserialize, Serialize)]
pub struct AnswerWithUserForm {
    pub answer: AnswerForm,
    pub user: UserForm,
}

#[post("/user")]
pub async fn sing_up(user: Json<UserForm>, pool: Data<DbPool>) -> Result<HttpResponse> {
    let user = user.into_inner();
    info!(
        "There is a new user:[name - {}, second name - {}]",
        user.name, user.name
    );

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let check_passed = block(move || db::check_if_user_exists(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Checking if a user exists error - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish()
        })?;

    if check_passed {
        debug!(
            "The [{} {}] user already present in the DB",
            user.name, user.second_name
        );
        return Ok(HttpResponse::AlreadyReported().finish());
    }

    let user_clone = user.clone();
    block(move || db::registry_new_user(user_clone, pool))
        .await
        .map_err(|err| {
            error!(
                "{}:{} An error occurred while registering a new user - {:?}",
                file!(),
                line!(),
                err
            );
            HttpResponse::InternalServerError().finish()
        })?;

    debug!("Successfully registry {} {} user", user.name, user.second_name);

    let UserForm { name, second_name, .. } = user;

    let http_response = HttpResponse::Created().content_type("application/json").json(json!({
        "name" : name,
        "second_name" : second_name,
        "scores" : 0
    }));

    Ok(http_response)
}

#[get("/user")]
pub async fn sing_in(user: Json<UserForm>, pool: Data<DbPool>) -> Result<HttpResponse> {
    let user = user.into_inner();
    debug!(
        "Validation of user:[name - {}, second name - {}]",
        user.name, user.second_name
    );

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let check_passed = block(move || db::check_if_user_exists(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Checking if a user exists error - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish()
        })?;

    if !check_passed {
        debug!("The user doesn't present in the DB");
        return Ok(HttpResponse::BadRequest().finish());
    }

    debug!("The {} {} user exists in the DB", user.name, user.second_name);
    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let verify_password_passed = block(move || db::verify_password(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Verifying an user password failed - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish()
        })?;

    if !verify_password_passed {
        debug!("The user hasn't passed password verify");
        return Ok(HttpResponse::Forbidden().finish());
    }
    debug!("The user passed password verifying");

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let scores = block(move || db::get_scores(&user_clone, &pool_clone))
        .await
        .map_err(|err| {
            error!(
                "{}:{} An error occurred while getting a user scores - {:?}",
                file!(),
                line!(),
                err
            );
            HttpResponse::InternalServerError().finish()
        })?;

    let UserForm { name, second_name, .. } = user;

    let http_response = HttpResponse::Found().content_type("application/json").json(json!({
       "name"  : name,
       "second_name": second_name,
       "scores"     : scores,
    }));

    Ok(http_response)
}

#[get("/test")]
pub async fn get_test(pool: Data<DbPool>) -> Result<HttpResponse> {
    let test = block(move || db::get_test(pool)).await.map_err(|err| {
        error!("{}:{} Getting a test failed - {:?}", file!(), line!(), err);
        HttpResponse::InternalServerError().finish()
    })?;

    let mut image = None;
    if test.image.is_some() {
        let image_base64 = base64::encode_block(test.image.unwrap().as_bytes());
        image = Some(image_base64);
    }

    let answers = serde_json::from_str::<Vec<String>>(&test.answers)?;
    let response = HttpResponse::Ok().content_type("application/json").json(json!({
        "id": test.id,
        "description": test.description,
        "answers"    : answers,
        "image": image,
    }));

    Ok(response)
}

#[get("/check_answer")]
pub async fn check_answer(query_data: Query<AnswerForm>, pool: Data<DbPool>) -> Result<HttpResponse> {
    let AnswerForm { test_id, answer_id } = query_data.into_inner();

    let check_result = block(move || db::check_test_answer(test_id, answer_id, &pool))
        .await
        .map_err(|err| {
            error!("{}:{} Checking a test answer failed - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish();
        })?;

    let json_data = match check_result {
        true => json!({
            "description": "The answer is correct",
            "scores": SCORES_FOR_RIGHT_ANSWER,
        }),
        false => json!({
            "description": "The answer is incorrect",
            "scores": 0
            ,
        }),
    };

    Ok(HttpResponse::Ok().content_type("application/json").json(json_data))
}

#[post("/check_test")]
pub async fn check_answer_with_user(user_data: Json<AnswerWithUserForm>, pool: Data<DbPool>) -> Result<HttpResponse> {
    let AnswerWithUserForm { user, answer } = user_data.into_inner();
    let AnswerForm { test_id, answer_id } = answer;

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let check_passed = block(move || db::check_if_user_exists(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Checking if a user exists error - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish();
        })?;

    if !check_passed {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let user_clone = user.clone();
    let pool_clone = pool.clone();
    let verify_passed = block(move || db::verify_password(user_clone, pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Verifying an user password failed - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish();
        })?;

    if !verify_passed {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let pool_clone = pool.clone();
    let check_result = block(move || db::check_test_answer(test_id, answer_id, &pool_clone))
        .await
        .map_err(|err| {
            error!("{}:{} Checking a test answer failed - {:?}", file!(), line!(), err);
            HttpResponse::InternalServerError().finish();
        })?;

    let json_data = match check_result {
        true => {
            let user_clone = user.clone();
            let pool_clone = pool.clone();

            block(move || db::add_scores(&user_clone, SCORES_FOR_RIGHT_ANSWER, &pool_clone))
                .await
                .map_err(|err| {
                    error!("{}:{} Failed to add new scores - {:?}", file!(), line!(), err);
                    HttpResponse::InternalServerError().finish();
                })?;

            json!({
                "description": "The answer is correct",
                "scores": SCORES_FOR_RIGHT_ANSWER,
            })
        }
        false => json!({
            "description": "The answer is incorrect",
            "scores": 0,
        }),
    };

    Ok(HttpResponse::Ok().content_type("application/json").json(json_data))
}

#[get("/healthy")]
pub async fn healthy() -> HttpResponse {
    let response = "Drive-tests is working and healthy".to_string();
    HttpResponse::Ok().content_type("application/json").json(response)
}
