use serde::{Deserialize, Serialize};

use crate::db::schema::{tests, users};
use std::io::{self, ErrorKind};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserForm {
    pub name: String,
    #[serde(rename = "second_name")]
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
    pub level: i32,
    pub description: String,
    pub answers: String, // TODO: replace with real JSON value
    pub right_answer_id: i32,
}

#[derive(PartialEq, Debug)]
pub enum TestLevel {
    Easy,
    Medium,
    High,
}

impl TestLevel {
    pub fn new(level_value: u32) -> Result<TestLevel, io::Error> {
        match level_value {
            1 => Ok(TestLevel::Easy),
            3 => Ok(TestLevel::Medium),
            5 => Ok(TestLevel::High),
            _ => Err(io::Error::new(ErrorKind::InvalidData, "Got incorrect test level value")),
        }
    }
    pub fn to_scores(&self) -> u32 {
        match self {
            TestLevel::Easy => 1,
            TestLevel::Medium => 3,
            TestLevel::High => 5,
        }
    }
}
