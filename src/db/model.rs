use serde_derive::Deserialize;

//use diesel::deserialize::{Queryable, QueryableByName};

use crate::db::schema::{tests, users};
use std::io::{self, ErrorKind};

#[derive(Deserialize, Insertable)]
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

#[derive(Deserialize, Insertable)]
#[table_name = "tests"]
pub struct TestFrom {
    id: i32,
    level: i32,
}

#[derive(Queryable, PartialEq, Debug)]
pub struct Tests {
    id: i32,
    level: TestLevel,
}

#[derive(PartialEq, Debug)]
pub enum TestLevel {
    Easy,
    Medium,
    High,
}

impl TestLevel {
    fn new(level_value: u32) -> Result<TestLevel, io::Error> {
        match level_value {
            1 => Ok(TestLevel::Easy),
            3 => Ok(TestLevel::Medium),
            5 => Ok(TestLevel::High),
            _ => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Got incorrect test level value",
            )),
        }
    }
}
