use actix_web::{
    body::Body,
    http::{header::ContentType, StatusCode},
    test::{call_service, init_service, TestRequest},
    web, App, HttpResponse,
};
use lazy_static::lazy_static;

use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use image::EncodableLayout;
use serde::Deserialize;
use uuid::Uuid;

use openssl::base64;
use std::ops::Deref;

use lib::{
    db::{
        model::{self, UserForm},
        remove_user_from_db,
    },
    web::{AnswerForm, AnswerWithUserForm},
    *,
};

lazy_static! {
    static ref DB: db::DbPool = db::establish_connection();
}

#[derive(Deserialize)]
struct TestResponseForm {
    #[allow(dead_code)]
    description: String,
    scores: i32,
}

impl TestResponseForm {
    fn from_http_response(mut response: HttpResponse) -> TestResponseForm {
        let body = response.take_body();

        match body.as_ref().unwrap() {
            Body::Bytes(data) => serde_json::from_slice::<TestResponseForm>(data).unwrap(),
            _ => panic!("Got an expected body from get test request"),
        }
    }
}

#[derive(Deserialize)]
struct UserResponseForm {
    name: String,
    second_name: String,
    scores: u32,
}

impl UserResponseForm {
    fn from_http_response(mut response: HttpResponse) -> UserResponseForm {
        let body = response.take_body();

        match body.as_ref().unwrap() {
            Body::Bytes(data) => serde_json::from_slice::<UserResponseForm>(data).unwrap(),
            _ => panic!("Got an expected body from get test request"),
        }
    }
}

#[derive(Deserialize)]
struct TestForm {
    id: i32,
    #[allow(dead_code)]
    description: String,
    answers: Vec<String>,
    image: Option<String>,
}

impl TestForm {
    fn from_http_response(mut response: HttpResponse) -> TestForm {
        let body = response.take_body();

        match body.as_ref().unwrap() {
            Body::Bytes(data) => {
                let json_test = serde_json::from_slice::<TestForm>(data).unwrap();

                if json_test.image.is_some() {
                    let image_base64 = json_test.image.as_ref().unwrap();
                    let image_data = base64::decode_block(image_base64).unwrap();
                    let _ = image::load_from_memory(image_data.as_bytes()).unwrap();
                }
                json_test
            }
            _ => panic!("Got an expected body from get test request"),
        }
    }
}

fn get_correct_answer_id_from_test_id(test_id: i32) -> u32 {
    use lib::db::schema::tests::dsl::*;

    let db = DB.get().unwrap();
    let selected_test: model::Test = tests
        .order(id)
        .filter(id.eq(test_id))
        .first::<model::Test>(db.deref())
        .unwrap();

    selected_test.right_answer_id as u32
}

fn get_user_scores(user: &UserForm) -> u32 {
    use lib::db::schema::users::dsl::*;

    let db = DB.get().unwrap();
    let selected_user: model::User = users
        .order(id)
        .filter(name.eq(user.name.clone()).and(second_name.eq(user.second_name.clone())))
        .first::<model::User>(db.deref())
        .unwrap();

    selected_user.scores as u32
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
    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user.clone(), &web::Data::new(DB.clone()));

    assert_eq!(response.status(), StatusCode::CREATED);

    let user_response_form = UserResponseForm::from_http_response(response.into());

    assert_eq!(user_response_form.name, user.name);
    assert_eq!(user_response_form.second_name, user.second_name);
    assert_eq!(user_response_form.scores, 0);
}

#[actix_rt::test]
async fn log_in_after_create_the_user() {
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
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user.clone(), &web::Data::new(DB.clone()));

    assert_eq!(response.status(), StatusCode::FOUND);

    let user_response_form = UserResponseForm::from_http_response(response.into());

    assert_eq!(user_response_form.name, user.name);
    assert_eq!(user_response_form.second_name, user.second_name);
    assert_eq!(user_response_form.scores, 0);
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

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
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

#[actix_rt::test]
async fn get_test_returns_correct_test() {
    let mut app = init_service(App::new().data(DB.clone()).service(get_test)).await;

    let request = TestRequest::get().uri("/test").to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());
    let _ = TestForm::from_http_response(response.into());
}

#[actix_rt::test]
async fn check_answer_returns_right_scores_if_a_test_passed() {
    let mut app = init_service(App::new().data(DB.clone()).service(get_test).service(check_answer)).await;

    let request = TestRequest::get().uri("/test").to_request();
    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let test_from = TestForm::from_http_response(response.into());

    let correct_answer_id = get_correct_answer_id_from_test_id(test_from.id);

    let url = format!("/check_answer?test_id={}&answer_id={}", test_from.id, correct_answer_id);
    let request = TestRequest::get().uri(&url).to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    assert_ne!(TestResponseForm::from_http_response(response.into()).scores, 0);
}

#[actix_rt::test]
async fn check_answer_return_zero_for_a_failed_test() {
    let mut app = init_service(App::new().data(DB.clone()).service(get_test).service(check_answer)).await;

    let request = TestRequest::get().uri("/test").to_request();
    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let test_from = TestForm::from_http_response(response.into());

    let correct_answer_id = get_correct_answer_id_from_test_id(test_from.id);

    let mut bad_answer_id = (rand::random::<usize>() % test_from.answers.len()) as u32;
    while bad_answer_id == correct_answer_id {
        bad_answer_id = (rand::random::<usize>() % test_from.answers.len()) as u32;
    }

    let url = format!("/check_answer?test_id={}&answer_id={}", test_from.id, bad_answer_id);
    let request = TestRequest::get().uri(&url).to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let test_response_form = TestResponseForm::from_http_response(response.into());
    assert_eq!(test_response_form.scores, 0);
}

#[actix_rt::test]
async fn check_answer_with_a_user_save_new_scores_correctly_for_a_passed_test() {
    let mut app = init_service(
        App::new()
            .data(DB.clone())
            .service(sing_in)
            .service(sing_up)
            .service(get_test)
            .service(check_answer_with_user),
    )
    .await;

    let user = create_rand_user();

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let request = TestRequest::get().uri("/test").to_request();
    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let test_from = TestForm::from_http_response(response.into());

    let correct_answer_id = get_correct_answer_id_from_test_id(test_from.id);

    let answer_with_user = AnswerWithUserForm {
        answer: AnswerForm {
            test_id: test_from.id as u32,
            answer_id: correct_answer_id,
        },
        user: user.clone(),
    };

    let request = TestRequest::post()
        .set_json(&answer_with_user)
        .uri(&"/check_test")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    assert_ne!(get_user_scores(&user), 0);

    remove_user_from_db(user, &web::Data::new(DB.clone()));
}

#[actix_rt::test]
async fn check_answer_with_a_user_for_a_failed_test_doesnt_change_scores() {
    let mut app = init_service(
        App::new()
            .data(DB.clone())
            .service(sing_in)
            .service(sing_up)
            .service(get_test)
            .service(check_answer_with_user),
    )
    .await;

    let user = create_rand_user();

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let request = TestRequest::get().uri("/test").to_request();
    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let test_from = TestForm::from_http_response(response.into());

    let correct_answer_id = get_correct_answer_id_from_test_id(test_from.id);

    let mut bad_answer_id = (rand::random::<usize>() % test_from.answers.len()) as u32;
    while bad_answer_id == correct_answer_id {
        bad_answer_id = (rand::random::<usize>() % test_from.answers.len()) as u32;
    }

    let answer_with_user = AnswerWithUserForm {
        user: user.clone(),
        answer: AnswerForm {
            test_id: test_from.id as u32,
            answer_id: bad_answer_id,
        },
    };

    let request = TestRequest::post()
        .set_json(&answer_with_user)
        .uri(&"/check_test")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    assert_eq!(get_user_scores(&user), 0);

    remove_user_from_db(user, &web::Data::new(DB.clone()));
}

#[actix_rt::test]
async fn passing_many_tests_in_a_row_correctly_processing_scores() {
    let mut app = init_service(
        App::new()
            .data(DB.clone())
            .service(sing_in)
            .service(sing_up)
            .service(get_test)
            .service(check_answer_with_user),
    )
    .await;

    let user = create_rand_user();

    let request = TestRequest::post()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    assert!(response.status().is_success());

    let mut scores = 0u32;
    for _ in 0..10 {
        let request = TestRequest::get().uri("/test").to_request();
        let response = call_service(&mut app, request).await;

        let test_from = TestForm::from_http_response(response.into());

        let rand_answer = (rand::random::<usize>() % test_from.answers.len()) as u32 + 1;

        let answer_with_user = AnswerWithUserForm {
            user: user.clone(),
            answer: AnswerForm {
                test_id: test_from.id as u32,
                answer_id: rand_answer,
            },
        };

        let request = TestRequest::post()
            .set_json(&answer_with_user)
            .uri(&"/check_test")
            .to_request();

        let response = call_service(&mut app, request).await;
        assert!(response.status().is_success());

        scores += TestResponseForm::from_http_response(response.into()).scores as u32;
    }

    let request = TestRequest::get()
        .set(ContentType::json())
        .set_json(&user)
        .uri("/user")
        .to_request();

    let response = call_service(&mut app, request).await;

    remove_user_from_db(user, &web::Data::new(DB.clone()));

    let mut response: HttpResponse = response.into();
    let body = response.take_body();

    match body.as_ref().unwrap() {
        Body::Bytes(data) => {
            let user_log_in_response = serde_json::from_slice::<UserResponseForm>(data).unwrap();
            assert_eq!(user_log_in_response.scores, scores);
        }
        _ => panic!("Got an expected body from get test request"),
    }
}
