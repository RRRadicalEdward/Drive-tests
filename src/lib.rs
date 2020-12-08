#[macro_use]
extern crate diesel;
#[macro_use]
extern crate anyhow;

pub mod db;
pub mod web;

pub use db::model;
