use serde::{Deserialize, Serialize};

//use diesel::deserialize::{Queryable, QueryableByName};

use crate::db::schema::{tests, users};
use std::io::{self, ErrorKind};

#[derive(Serialize, Deserialize, Insertable, Clone)]
#[table_name = "users"]
pub struct UserForm {
    pub name: String,
    pub second_name: String,
    pub password: String,
}
#[derive(Queryable, PartialEq, Debug)]
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
    pub answers: String,
}

#[derive(Queryable, PartialEq, Debug)]
pub struct Tests {
    id: i32,
    level: TestLevel,
    description: String,
    answers: String,
}

#[derive(PartialEq, Debug)]
pub enum TestLevel {
    Easy,
    Medium,
    High,
}

impl TestLevel {
    #![allow(dead_code)]
    fn new(level_value: u32) -> Result<TestLevel, io::Error> {
        match level_value {
            1 => Ok(TestLevel::Easy),
            3 => Ok(TestLevel::Medium),
            5 => Ok(TestLevel::High),
            _ => Err(io::Error::new(ErrorKind::InvalidData, "Got incorrect test level value")),
        }
    }
}
