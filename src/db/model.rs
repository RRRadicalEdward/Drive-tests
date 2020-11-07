use serde_derive::Deserialize;

//use diesel::deserialize::{Queryable, QueryableByName};

use crate::db::schema::users;
use std::io::{self, ErrorKind};

#[derive(Deserialize, Insertable)]
#[table_name = "users"]
pub struct UserForm<'a> {
    name: &'a str,
    second_name: &'a str,
    password: &'a str,
}
#[derive(Queryable, PartialEq, Debug)]
pub struct User {
    id: i32,
    name: String,
    second_name: String,
    password: String,
    scores: i32,
}
#[derive(Queryable, PartialEq, Debug)]
pub struct Tests {
    id: u32,
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
