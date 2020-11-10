#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

pub mod db;
pub mod web;

use db::model;
