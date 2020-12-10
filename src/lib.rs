#[macro_use]
extern crate diesel;
#[macro_use]
extern crate anyhow;

#[allow(unused_imports)] // TODO: Research how to fix without unused_import
#[macro_use]
extern crate lazy_static; // It is used in tests cfg(test) mod.

pub mod db;
pub mod web;

pub use db::model;
