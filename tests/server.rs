use lazy_static::lazy_static;

use actix_web::{test, web, App, body::Body, HttpResponse, http};

use uuid::Uuid;
use serde_json::json;

use lib::*;
use lib::db::model::UserForm;

lazy_static! {
    static ref DB: db::DbPool = db::establish_connection();
}

#[actix_rt::test]
async fn test_request() {
    let mut app = test::init_service(App::new().data(DB.clone()).service(lib::web::healthy)).await;

    let request = test::TestRequest::get().uri("/healthy").to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn create_user(){
    let mut  app = test::init_service(
        App::new()
            .data(DB.clone())
            .service(lib::web::sing_in)
    ).await;

    let name = Uuid::new_v4().to_string();
    let second_name = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();

    let expected_json_body = json!({
          "name" : name.clone(),
           "second_name" : second_name.clone(),
           "scores" : "0",
    });

    let user_form  = UserForm{name, second_name, password};
    let json_data = serde_json::to_string(&user_form).unwrap();
    let request = test::TestRequest::post().set(http::header::ContentType::json()).set_json(&json_data).uri("/user").to_request();
      //  .send_request(&mut app).await;
    let response = test::call_service(&mut app, request).await;

    lib::db::remove_user_from_db(user_form, &web::Data::new(DB.clone()));

    assert_eq!(response.status(), actix_web::http::StatusCode::OK);

    let mut response: HttpResponse = response.into();
    let body = response.take_body();
    let body = body.as_ref().unwrap();

    assert_eq!(Body::from(expected_json_body), *body);
}
