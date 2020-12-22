use serde::{Deserialize, Serialize};

use crate::db::schema::{tests, users};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
pub struct UserForm {
    pub name: String,
    pub second_name: String,
    pub password: String,
}

#[derive(Queryable, PartialEq, Debug, Deserialize, Insertable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub name: String,
    pub second_name: String,
    pub password: String,
    pub scores: i32,
}

#[derive(Queryable, PartialEq, Debug, Deserialize, Insertable)]
#[table_name = "tests"]
pub struct Test {
    pub id: i32,
    pub description: String,
    pub answers: String,
    pub image: Vec<u8>,
    pub right_answer_id: i32,
}
