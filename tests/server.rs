use lazy_static::lazy_static;

use actix_web::{
    body::Body,
    http::{header::ContentType, StatusCode},
    test::{call_service, init_service, TestRequest},
    web, App, HttpResponse,
};

use serde_json::json;
use uuid::Uuid;

use lib::{
    db::{model::UserForm, remove_user_from_db},
    *,
};

lazy_static! {
    static ref DB: db::DbPool = db::establish_connection();
}

fn create_rand_user() -> UserForm {
    let name = Uuid::new_v4().to_string();
    let second_name = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    UserForm {
        name,
        second_name,
        password,
    }
}

#[actix_rt::test]
async fn test_request() {
    let mut app = init_service(App::new().data(DB.clone()).service(healthy)).await;

    let request = TestRequest::get().uri("/healthy").to_request();
    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn create_user() {
    let mut app = init_service(App::new().data(DB.clone()).service(sing_up)).await;

    let user = create_rand_user();
    let expected_json_body = json!({
          "name" : user.name.clone(),
           "second_name" : user.second_name.clone(),
           "scores" : "0",
    });

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user, &web::Data::new(DB.clone()));

    assert_eq!(response.status(), StatusCode::CREATED);

    let mut response: HttpResponse = response.into();
    let body = response.take_body();
    let body = body.as_ref().unwrap();

    assert_eq!(*body, Body::from(expected_json_body));
}

#[actix_rt::test]
async fn log_in_after_create_the_user() {
    let mut app = init_service(App::new().data(DB.clone()).service(sing_up).service(sing_in)).await;

    let user = create_rand_user();

    let expected_json_body = json!({
          "name" : user.name.clone(),
           "second_name" : user.second_name.clone(),
           "scores" : 0,
    });

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let _ = call_service(&mut app, request).await;

    let request = TestRequest::get()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user, &web::Data::new(DB.clone()));

    assert_eq!(response.status(), StatusCode::FOUND);

    let mut response: HttpResponse = response.into();
    let body = response.take_body();
    let body = body.as_ref().unwrap();

    assert_eq!(*body, Body::from(expected_json_body));
}

#[actix_rt::test]
async fn log_in_for_not_existing_user() {
    let mut app = init_service(App::new().data(DB.clone()).service(sing_in)).await;

    let user = create_rand_user();

    let request = TestRequest::get()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[actix_rt::test]
async fn log_in_for_bad_password() {
    let mut app = init_service(App::new().data(DB.clone()).service(sing_up).service(sing_in)).await;

    let user = create_rand_user();

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let _ = call_service(&mut app, request).await;

    let request = TestRequest::get()
        .set(ContentType::json())
        .set_json(&UserForm {
            name: user.name.clone(),
            second_name: user.second_name.clone(),
            password: "SomeBadPassword".to_string(),
        })
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user, &web::Data::new(DB.clone()));

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
